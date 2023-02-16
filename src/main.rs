#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use std::{
    env,
    io::{
        prelude::*,
        stdin,
        stdout
    },
    str::FromStr,
    string::ToString
};
use quantum_werewolf::{
    game::{
        self,
        Role,
        state::Signups
    },
    handler::CliHandler,
    player::CliPlayer
};

struct Args {
    roles: Option<Vec<Role>>
}

impl Args {
    fn set_roles(&mut self, roles: &str) {
        self.roles = Some(
            roles
                .split(',')
                .map(|role_str| Role::from_str(role_str).expect("no such role"))
                .collect()
        );
    }
}

impl Default for Args {
    fn default() -> Args {
        Args {
            roles: None
        }
    }
}

fn args() -> Args {
    enum ArgsMode {
        Roles
    }

    let mut args = Args::default();
    let mut mode = None;
    for arg in env::args().skip(1) {
        match mode {
            Some(ArgsMode::Roles) => { args.set_roles(&arg); }
            None => {
                if arg != "-" && arg.starts_with('-') {
                    // flags and options
                    if arg.starts_with("--") {
                        if arg == "--roles" {
                            mode = Some(ArgsMode::Roles);
                        } else if arg.starts_with("--roles=") {
                            args.set_roles(&arg["--roles=".len()..]);
                        } else {
                            panic!("unrecognized flag: {:?}", arg);
                        }
                    } else {
                        for (i, short_flag) in arg.chars().enumerate() {
                            if i == 0 { continue; }
                            panic!("unrecognized flag: -{:?}", short_flag);
                        }
                    }
                } else {
                    // positional args
                    panic!("unexpected positional argument: {:?}", arg);
                }
            }
        }
    }
    args
}

fn join<S: ToString, I: IntoIterator<Item=S>>(words: I) -> String {
    let mut words = words.into_iter().map(|word| word.to_string()).collect::<Vec<_>>();
    match words.len() {
        0 => "no one".to_owned(),
        1 => format!("{}", words.swap_remove(0)),
        2 => format!("{} and {}", words.swap_remove(0), words.swap_remove(0)),
        _ => {
            let last = words.pop().unwrap();
            format!("{}, and {}", words.join(", "), last)
        }
    }
}

fn main() {
    let args = args();
    let mut game_state = Signups::default();
    loop {
        print!("[ ?? ] player name [leave blank to finish]: ");
        stdout().flush().expect("failed to flush stdout");
        let mut name = String::new();
        stdin().read_line(&mut name).expect("failed to read username");
        assert_eq!(name.pop(), Some('\n'));
        if name.is_empty() {
            break;
        }
        if !game_state.sign_up(CliPlayer::from(name)) {
            println!("[ !! ] duplicate player name");
        }
    }
    let winners = if let Some(roles) = args.roles {
        game::run_with_roles(CliHandler, game_state, roles)
    } else {
        game::run(CliHandler, game_state)
    }.expect("failed to start game");
    println!("[ ** ] The winners are: {}", join(winners));
}
