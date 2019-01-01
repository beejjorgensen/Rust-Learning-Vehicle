extern crate rand;

use dungeon::Dungeon;
use player::Player;
use room::RoomType;
use treasure::Treasure;
use monster::{Monster,MonsterType};
use weapon::{Weapon,WeaponType};
use error::Error;

use self::rand::Rng;
use self::rand::thread_rng;

#[derive(Debug,Clone)]
pub enum Event {
    None,
    FoundGold(usize),
    FoundFlares(usize),
    Sinkhole,
    Warp,
    Treasure(Treasure),
    Combat(MonsterType),
}

#[derive(Debug,Clone,Copy)]
pub enum CombatEvent {
    NoWeapon,
    //BookHands,
    Miss,
    Hit(usize, bool, bool, usize, bool),
    MonsterMiss,
    MonsterHit(usize, bool, bool),
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum GameState {
    Init,

    Move,

    Vendor,

    PlayerAttack,
    MonsterAttack,
    Retreat,

    Warp,
    Sinkhole,
    Gas,

    Dead,
    Exit,
}

pub struct Game {
    pub dungeon: Dungeon,
    pub player: Player,
    state: GameState,
    prev_dir: Direction,
    currently_fighting: Option<Monster>,
    bribe_possible: bool,
    retreating:bool,
}

impl Game {
    pub fn new(xsize: usize, ysize: usize, zsize: usize) -> Game {

        let dungeon = Dungeon::new(xsize, ysize, zsize);

        let mut player = Player::new();
        player.set_position(dungeon.entrance_x(), 0, 0);

        Game {
            dungeon,
            player,
            state: GameState::Init,
            prev_dir: Direction::South,
            currently_fighting: None,
            bribe_possible: true,
            retreating: false,
        }
    }
    
    /// Mark the player's current room as empty
    fn make_current_room_empty(&mut self) {
        let room = self.dungeon.room_at_mut(self.player.x, self.player.y, self.player.z);

        room.make_empty();
    }

    /// Handle Gold room effects
    fn room_effect_gold(&mut self) -> Event {
        let gold_amount = Game::d(1,10);

        self.player.gp += gold_amount;

        self.make_current_room_empty();

        return Event::FoundGold(gold_amount);
    }

    /// Handle Flare room effects
    fn room_effect_flares(&mut self) -> Event {
        let flare_amount = Game::d(1,5);

        self.player.flares += flare_amount;

        self.make_current_room_empty();

        return Event::FoundFlares(flare_amount);
    }

    /// Handle Sinkhole room effects
    fn room_effect_sinkhole(&mut self) -> Event {
        self.player.z = (self.player.z + 1) % self.dungeon.zsize;

        return Event::Sinkhole;
    }

    /// Handle Warp room effects
    fn room_effect_warp(&mut self, orb_of_zot:bool) -> Event {
        if orb_of_zot {
            let prev_dir = self.prev_dir;
            self.move_dir(prev_dir);
        } else {
            let mut rng = thread_rng();

            self.player.x = rng.gen_range(0, self.dungeon.xsize);
            self.player.y = rng.gen_range(0, self.dungeon.ysize);
            self.player.z = rng.gen_range(0, self.dungeon.zsize);
        }

        return Event::Warp;
    }

    /// Handle Treasure room effects
    fn room_effect_treasure(&mut self, treasure:Treasure) -> Event {
        self.make_current_room_empty();

        self.player.treasures.push(treasure.treasure_type);

        Event::Treasure(treasure)
    }

    // Handle Monster room effects
    fn room_effect_monster(&mut self, monster:Monster) -> Event {
        self.currently_fighting = Some(monster.clone());

        // TODO check for blind or lethargy

        self.state = GameState::PlayerAttack;

        self.bribe_possible = true;

        self.retreating = false;

        Event::Combat(monster.monster_type())
    }

    /// True if the player can bribe
    pub fn bribe_possible(&self) -> bool {
        self.bribe_possible
    }

    /// True if the player can cast a spell
    pub fn spell_possible(&self) -> bool {
        self.player.iq > 14
    }

    /// Handle player attacking monster
    pub fn attack(&mut self) -> Result<CombatEvent, Error> {
        if self.state != GameState::PlayerAttack {
            return Err(Error::WrongState);
        }

        if self.player.weapon.weapon_type() == WeaponType::None {
            self.state = GameState::MonsterAttack;
            return Ok(CombatEvent::NoWeapon);
        }

        let hit = self.player.dx >= (Game::d(1, 20) + (self.player.is_blind() as usize) * 3);

        if hit {
            let damage = self.player.weapon.damage();
            let mut broke_weapon = false;
            let mut next_state = GameState::MonsterAttack;
            let defeated;
            let mut got_runestaff = false;
            let treasure;

            if let Some(ref mut monster) = self.currently_fighting {
                if monster.can_break_weapon() && Game::d(1,8) == 1 {
                    broke_weapon = true;
                    self.player.weapon = Weapon::new(WeaponType::None);
                }

                defeated = monster.take_damage(damage);
                
                if defeated {
                    next_state = GameState::Move;

                    if monster.has_runestaff() {
                        self.player.receive_runestaff();
                        got_runestaff = true;
                    }
                }
            } else {
                panic!("not fighting a monster");
            }

            if defeated {
                self.make_current_room_empty();
                self.currently_fighting = None;

                treasure = Game::d(1,1000);
            } else {
                treasure = 0;
            }

            self.player.gp += treasure;

            self.state = next_state;
            return Ok(CombatEvent::Hit(damage, broke_weapon, defeated, treasure, got_runestaff));
        }

        self.state = GameState::MonsterAttack;
        Ok(CombatEvent::Miss)
    }

    /// Helper function to get the next state after a monster attack
    fn state_after_monster_attack(&mut self) {
        if self.retreating {
            self.state = GameState::Retreat;
        } else {
            self.state = GameState::PlayerAttack;
        }
    }

    /// Handle a monster attack
    pub fn be_attacked(&mut self) -> Result<CombatEvent, Error> {
        if self.state != GameState::MonsterAttack {
            return Err(Error::WrongState);
        }

        // TODO check for web breaking

        // TODO check for stuck in web

        let hit = self.player.dx < (Game::d(3,7) + (self.player.is_blind() as usize) * 3);

        let mut combat_event = None;
        let mut defeated = false;

        // Handle player hit
        if hit {
            if let Some(ref mut monster) = self.currently_fighting {
                let damage = monster.damage();
                let armor_value = self.player.armor().armor_value();

                let st_damage = std::cmp::max(damage - armor_value, 0);
                defeated = self.player.damage_st(st_damage);

                let armor_damage = std::cmp::min(damage, armor_value);
                let armor_destroyed = self.player.damage_armor(armor_damage);

                combat_event = Some(CombatEvent::MonsterHit(st_damage, defeated, armor_destroyed));

            } else {
                panic!("being attacked, but not by any monster");
            }
        }

        // Set next state
        if hit {
            if defeated {
                self.state = GameState::Dead;
            } else {
                self.state_after_monster_attack();
            }

            if let Some(c_event) = combat_event {
                return Ok(c_event);
            }
        }

        self.state_after_monster_attack();

        Ok(CombatEvent::MonsterMiss)
    }

    /// Handle retreat
    ///
    /// This is split out from retreat_dir because the monster gets another
    /// attack in the middle of it.
    pub fn retreat(&mut self) -> Result<(), Error> {
        if self.state != GameState::PlayerAttack {
            return Err(Error::WrongState);
        }

        self.state = GameState::MonsterAttack;
        self.retreating = true;

        Ok(())
    }

    /// After the monster's final attack
    pub fn retreat_dir(&mut self, dir:Direction) {
        self.state = GameState::Move;

        self.move_dir(dir);
    }
    
    /// Check for a room event
    pub fn room_effect(&mut self) -> Event {

        let roomtype;

        {
            let room = self.dungeon.room_at(self.player.x, self.player.y, self.player.z);
            roomtype = room.roomtype.clone();
        }

        match roomtype {
            RoomType::Gold => self.room_effect_gold(),
            RoomType::Flares => self.room_effect_flares(),
            RoomType::Sinkhole => self.room_effect_sinkhole(),
            RoomType::Warp(orb_of_zot) => self.room_effect_warp(orb_of_zot),
            RoomType::Treasure(t) => self.room_effect_treasure(t),
            RoomType::Monster(m) => self.room_effect_monster(m),
            _ => Event::None,
        }
    }

    /// Handle a move command
    pub fn move_dir(&mut self, dir:Direction) {
        let p = &mut self.player;

        let xsize = self.dungeon.xsize;
        let ysize = self.dungeon.ysize;

        let room = self.dungeon.room_at(p.x, p.y, p.z);

        self.prev_dir = dir;

        // Handle exit special case
        if room.roomtype == RoomType::Entrance && dir == Direction::North {
            self.state = GameState::Exit;
            return;
        }

        match dir {
            Direction::North => {
                if p.y == 0 {
                    p.y = ysize - 1;
                } else {
                    p.y -= 1;
                }
            }
            Direction::South => p.y = (p.y + 1) % ysize,
            Direction::West =>  {
                if p.x == 0 {
                    p.x = xsize - 1;
                } else {
                    p.x -= 1;
                }
            }
            Direction::East => p.x = (p.x + 1) % xsize,
        }
    }

    /// Roll a die (1d6, 2d7, etc.)
    pub fn d(count:usize, sides:usize) -> usize {
        let mut total = 0;

        let mut rng = thread_rng();

        for _ in 0..count {
            total += rng.gen_range(0, sides) + 1;
        }

        total
    }

    /// Return game state
    pub fn state(&self) -> GameState {
        self.state
    }
}