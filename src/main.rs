#![warn(trivial_casts)]
#![forbid(unused, unused_extern_crates, unused_import_braces)]

extern crate quantum_werewolf;

use std::env;
use std::io::prelude::*;
use std::io::{stdin, stdout};
use std::str::FromStr;

use quantum_werewolf::game::{Game, Role};
use quantum_werewolf::player::{Player, CliPlayer};

fn main() {
    let args = env::args().collect::<Vec<_>>();
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
    let roles = args.iter().position(|arg| arg == "--roles").map(|pos|
        args[pos + 1]
            .split(',')
            .map(|role_str| Role::from_str(role_str).expect("no such role"))
            .collect()
    );
    let game = Game::new(players, roles).expect("failed to create game");
    println!("[ ** ] The winners are: {:?}", game.run());
}
