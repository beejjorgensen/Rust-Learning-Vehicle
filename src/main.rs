extern crate wizardscastle;
extern crate rand; 

use std::io::{stdin,stdout,Write};

use self::rand::Rng;
use self::rand::thread_rng;

use wizardscastle::game::Game;
use wizardscastle::room::RoomType;
use wizardscastle::player::{Race, Gender, Stat};
use wizardscastle::armor::ArmorType;
use wizardscastle::weapon::WeaponType;

struct UI {
    game: Game,
    turn_count: usize,
}

impl UI {
    /// Return a random monster name
    fn rand_monster_str() -> String {
        let name = [
            "kobold",
            "orc",
            "wolf",
            "goblin",
            "ogre",
            "troll",
            "bear",
            "minotaur",
            "gargoyle",
            "chimera",
            "balrog",
            "dragon",
        ];

        let mut rng = thread_rng();

        let i = rng.gen_range(0, name.len());

        String::from(name[i]).to_uppercase()
    }

    fn starts_with_vowel(s: &str) -> bool {
        if let Some(c) = String::from(s).to_uppercase().chars().next() {
            return c == 'A' || c == 'E' || c == 'I' || c == 'O' || c == 'U';
        }

        return false;
    }

    fn get_article(s: &str) -> String {
        if UI::starts_with_vowel(s) {
            return String::from("AN");
        }

        String::from("A")
    }

    /// Print a map
    fn map(&self, show_all: bool) {
        let z = self.game.player.z;

        for y in 0..self.game.dungeon.ysize {
            for x in 0..self.game.dungeon.xsize {

                if x >= 1 {
                    print!("   ");
                }

                let r = self.game.dungeon.room_at(x, y, z);

                let bracket = x == self.game.player.x && y == self.game.player.y;

                if bracket {
                    print!("<");
                } else {
                    print!(" ");
                }

                if r.discovered || show_all {
                    match r.roomtype {
                        RoomType::Empty => print!("."),
                        RoomType::Entrance => print!("E"),
                        RoomType::StairsDown => print!("D"),
                        RoomType::StairsUp => print!("U"),
                        RoomType::Gold => print!("G"),
                        RoomType::Pool => print!("P"),
                        RoomType::Chest => print!("C"),
                        RoomType::Flares => print!("F"),
                        RoomType::Warp(_) => print!("W"),
                        RoomType::Sinkhole => print!("S"),
                        RoomType::CrystalOrb => print!("O"),
                        RoomType::Book => print!("B"),
                        RoomType::Monster(_) => print!("M"),
                        RoomType::Treasure(_) => print!("T"),
                    }
                } else {
                    print!("?");
                }

                if bracket {
                    print!(">");
                } else {
                    print!(" ");
                }
            }

            println!("\n");
        }
    }

    fn race_str(&self) -> &str {
        match self.game.player.race {
            Race::Hobbit => "HOBBIT",
            Race::Elf => "ELF",
            Race::Human => "HUMAN",
            Race::Dwarf => "DWARF",
        }
    }

    /// Input a line of text
    fn get_input(prompt: Option<&str>) -> String {
        let mut s = String::new();

        if let Some(s) = prompt {
            print!("{}", s);
            stdout().flush().unwrap();
        }

        stdin().read_line(&mut s).expect("Input error");

        s.trim().to_string().to_uppercase()
    }

    /// Print intro text
    ///
    /// Note: the original version lacked this preamble--it only appears in the
    /// magazine article. It was, however, included in the MBASIC port.
    ///
    fn intro(&self) {
        println!("\n{:*^64}\n", "");

        println!("{:^64}\n", "* * * THE WIZARD'S CASTLE * * *");

        println!("{:*^64}\n", "");

        println!("MANY CYCLES AGO, IN THE KINGDOM OF N'DIC, THE GNOMIC");
        println!("WIZARD ZOT FORGED HIS GREAT *ORB OF POWER*. HE SOON");
        println!("VANISHED, LEAVING BEHIND HIS VAST SUBTERRANEAN CASTLE");
        println!("FILLED WITH ESURIENT MONSTERS, FABULOUS TREASURES, AND");
        println!("THE INCREDIBLE *ORB OF ZOT*. FROM THAT TIME HENCE, MANY");
        println!("A BOLD YOUTH HAS VENTURED INTO THE WIZARD'S CASTLE. AS");
        println!("OF NOW, *NONE* HAS EVER EMERGED VICTORIOUSLY! BEWARE!!\n");
    }

    /// Select the player's race and sex
    fn race_gender_select(&mut self) {
        let race = loop {

            println!("ALL RIGHT, BOLD ONE.");
            println!("YOU MAY BE AN ELF, DWARF, MAN, OR HOBBIT.\n");

            let race_str = UI::get_input(Some("YOUR CHOICE? "));

            match race_str.get(..1) {
                Some("H") => break Race::Hobbit,
                Some("E") => break Race::Elf,
                Some("M") => break Race::Human,
                Some("D") => break Race::Dwarf,
                _ => println!("** THAT WAS INCORRECT. PLEASE TYPE E, D, M, OR H.\n"),
            }
        };

        self.game.player.init(race);

        let gender = loop {
            let gender_str = UI::get_input(Some("\nWHICH SEX TO YOU PREFER? "));

            match gender_str.get(..1) {
                Some("M") => break Gender::Male,
                Some("F") => break Gender::Female,
                _ => println!("** CUTE {}, REAL CUTE. TRY M OR F.", self.race_str()),
            }
        };

        self.game.player.set_gender(gender);
    }

    /// Allocate additional stat points
    fn allocate_points(&mut self) {
        println!("\nOK {}, YOU HAVE THESE STATISTICS:\n", self.race_str());

        println!("STRENGTH= {} INTELLIGENCE= {} DEXTERITY= {}\n",
            self.game.player.st, self.game.player.iq, self.game.player.dx);

        println!("AND {} OTHER POINTS TO ALLOCATE AS YOU WISH.\n", self.game.player.additional_points);

        let stats = vec!(Stat::Intelligence, Stat::Strength, Stat::Dexterity);
        let stat_names = vec!("INTELLIGENCE", "STRENGTH", "DEXTERITY");

        for i in 0..3 {
            let mut ok = false;

            while !ok {
                let s = UI::get_input(Some(&format!("HOW MANY POINTS DO YOU ADD TO {}? ", stat_names[i])));

                let points_to_add;
                
                match s.parse::<usize>() {
                    Ok(p) => points_to_add = p,
                    Err(_) => {
                        print!("\n** ");
                        continue;
                    },
                };

                if let Ok(_) = self.game.player.allocate_points(&stats[i], points_to_add) {
                    ok = true;
                } else {
                    print!("\n** ");
                    continue;
                }
            }

            if self.game.player.additional_points == 0 {
                return;
            }
        }
    }

    /// Buy armor
    fn buy_armor(&mut self) {
        println!("\nOK, {}, YOU HAVE {} GOLD PIECES (GP's)\n", self.race_str(), self.game.player.gp);

        println!("HERE IS A LIST OF ARMOR YOU CAN BUY (WITH COST IN <>)\n");

        println!("PLATE<30> CHAINMAIL<20> LEATHER<10> NOTHING<0>");

        let _ = loop {
            let armor_str = UI::get_input(Some("\nYOUR CHOICE? "));

            match armor_str.get(..1) {

                Some("P") => break self.game.player.purchase_armor(ArmorType::Plate, false),
                Some("C") => break self.game.player.purchase_armor(ArmorType::Chainmail, false),
                Some("L") => break self.game.player.purchase_armor(ArmorType::Leather, false),
                Some("N") => break self.game.player.purchase_armor(ArmorType::None, false),
                _ => {
                    let mon_str = UI::rand_monster_str();
                    let article = UI::get_article(&mon_str);

                    println!("\n** ARE YOU A {} OR {} {}? TYPE P,C,L OR N", self.race_str(), article, mon_str);
                },
            }
        };
    }

    /// Buy weapon
    fn buy_weapon(&mut self) {

        println!("\nOK, BOLD {}, YOU HAVE {} GP's LEFT\n", self.race_str(), self.game.player.gp);

        println!("HERE IS A LIST OF WEAPONS YOU CAN BUY (WITH COST IN <>)\n");

        println!("SWORD<30> MACE<20> DAGGER<10> NOTHING<0>");

        let _ = loop {
            let armor_str = UI::get_input(Some("\nYOUR CHOICE? "));

            match armor_str.get(..1) {

                Some("S") => break self.game.player.purchase_weapon(WeaponType::Sword, false),
                Some("M") => break self.game.player.purchase_weapon(WeaponType::Mace, false),
                Some("D") => break self.game.player.purchase_weapon(WeaponType::Dagger, false),
                Some("N") => break self.game.player.purchase_weapon(WeaponType::None, false),
                _ => println!("\n** IS YOUR IQ REALLY {}? TYPE S, M, D, OR N", self.game.player.iq),
            }
        };
    }

    /// Buy lamp
    fn buy_lamp(&mut self) {
        if !self.game.player.can_purchase_lamp() {
            return;
        }

        let _ = loop {
            let lamp_str = UI::get_input(Some("\nWANT TO BUY A LAMP FOR 20 GP's? "));

            match lamp_str.get(..1) {
                Some("Y") => break self.game.player.purchase_lamp(true),
                Some("N") => break self.game.player.purchase_lamp(false),
                _ => println!("\n** ANSWER YES OR NO"),
            }
        };
    }

    /// Buy flares
    fn buy_flares(&mut self) {
        let max_flares = self.game.player.max_flares();

        if max_flares == 0 {
            return;
        }

        println!("\nOK, {}, YOU HAVE {} GOLD PIECES LEFT\n", self.race_str(), self.game.player.gp);

        loop {
            let flare_str = UI::get_input(Some("FLARES COST 1 GP EACH, HOW MANY DO YOU WANT? "));

            let flare_count;
            
            match flare_str.parse::<usize>() {
                Ok(f) => flare_count = f,
                Err(_) => {
                    print!("** IF YOU DON'T WANT ANY JUST TYPE 0 (ZERO)\n\n");
                    continue;
                },
            };

            match self.game.player.purchase_flares(flare_count) {
                Ok(_) => break,
                Err(_) => {
                    print!("** YOU CAN ONLY AFFORD {}\n\n", max_flares);
                    continue;
                }
            }
        };
    }

    /// Print the player's location
    ///
    /// Note: the original game had a horizontal Y axis and a vertical X axis.
    /// This version reverses that.
    ///
    fn print_location(&self) {
        let p = &self.game.player;

        if p.is_blind() {
            return;
        }

        println!("YOU ARE AT ({},{}) LEVEL {}\n", p.x, p.y, p.z);
    }

    /// Print player stats
    fn print_stats(&self) {
        let p = &self.game.player;

        println!("ST={} IQ={} DX={} FLARES={} GP's={}",
            p.stat(Stat::Strength),
            p.stat(Stat::Intelligence),
            p.stat(Stat::Dexterity),
            p.flares(),
            p.gp());
    }
/*
        1670 PRINT:IFBL=0THENGOSUB3400:PRINT
1680 PRINT"ST= ";ST;" IQ= ";IQ;" DX= ";DX;" FLARES= ";FL;" GP's= ";GP
1690 PRINT:PRINTW$(WV+1);" / ";W$(AV+5);:IFLF=1THENPRINT" / A LAMP";
1700 PRINT:PRINT:WC=0:Q=FNE(PEEK(FND(Z))):POKEFND(Z),Q:Z$="YOU HAVE "
1710 PRINT"HERE YOU FIND ";C$(Q):IF(Q<7)OR(Q=11)OR(Q=12)THEN620

3400 PRINT"YOU ARE AT (";X;",";Y;") LEVEL ";Z:RETURN
*/

}

/// Main
fn main() {
    let game = Game::new(8, 8, 8);

    let mut ui = UI {
        game: game,
        turn_count: 0,
    };

    ui.intro();

    ui.race_gender_select();
    ui.allocate_points();
    ui.buy_armor();
    ui.buy_weapon();
    ui.buy_lamp();
    ui.buy_flares();

    println!("\nOK {}, YOU ENTER THE CASTLE AND BEGIN.\n", ui.race_str());

    let playing = true;
    ui.turn_count = 0;

    while playing {
        ui.turn_count += 1;

        ui.game.dungeon.discover(ui.game.player.x, ui.game.player.y, ui.game.player.z);

        println!();

        ui.print_location();
        ui.print_stats();
        //ui.print_room();

        ui.map(false);
    }
}
