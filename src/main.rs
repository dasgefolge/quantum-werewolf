#![warn(trivial_casts)]
#![forbid(unused, unused_extern_crates, unused_import_braces)]

extern crate quantum_werewolf;

use std::io::prelude::*;
use std::io::{stdin, stdout};

use quantum_werewolf::Game;
use quantum_werewolf::player::{Player, CliPlayer};

fn main() {
    let mut players: Vec<Box<Player>> = vec![];
    loop {
        print!("[ ?? ] player name [leave blank to finish]: ");
        stdout().flush().expect("failed to flush stdout");
        let mut name = String::new();
        stdin().read_line(&mut name).expect("failed to read username");
        assert_eq!(name.pop(), Some('\n'));
        if name.is_empty() {
            break;
        }
        players.push(Box::new(CliPlayer::new(name)));
    }
    let game = Game::new(players).expect("failed to create game");
    println!("[ ** ] The winners are: {:?}", game.run());
}
