#![warn(trivial_casts)]
#![deny(unused)]
#![forbid(unused_extern_crates, unused_import_braces)]

extern crate console;
#[macro_use] extern crate lazy_static;
extern crate quantum_werewolf;

use std::env;
use std::io::prelude::*;
use std::io::{stdin, stdout};
use std::str::FromStr;

use console::Term;

use quantum_werewolf::game::{Game, Role};
use quantum_werewolf::player::Player;
use quantum_werewolf::player::cli::{Cli, CliPlayer};

lazy_static! {
    static ref TERM: Term = Term::stdout();
    static ref CLI: Cli<'static> = Cli::from(TERM);
}

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
        players.push(Box::new(CliPlayer::new(name, &CLI)));
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
