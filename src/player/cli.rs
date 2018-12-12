use std::{
    fmt,
    io::{
        prelude::*,
        stdin,
        stdout
    }
};
use crate::{
    game::Faction,
    player::Player
};

/// A player who sends game actions via the command line.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CliPlayer {
    name: String
}

impl CliPlayer {
    fn input_secret(&self, msg: &str) -> String {
        print!("[ ?? ] @{}: {}: ", self.name, msg);
        stdout().flush().expect("failed to flush stdout");
        let mut name = String::new();
        stdin().read_line(&mut name).expect("failed to read player input");
        assert_eq!(name.pop(), Some('\n'));
        name
    }

    fn print_secret(&self, msg: &str) {
        println!("[ __ ] @{}: {}", self.name, msg);
    }
}

impl From<String> for CliPlayer {
    /// Creates a new CLI player with the given player name.
    fn from(name: String) -> CliPlayer {
        CliPlayer { name }
    }
}

impl Player for CliPlayer {
    fn recv_id(&self, player_id: usize) {
        self.print_secret(&format!("your secret player ID is {}", player_id)[..]);
    }

    fn choose_heal_target(&self, possible_targets: Vec<&CliPlayer>) -> Option<CliPlayer> {
        loop {
            let result = CliPlayer::from(self.input_secret("player to heal"));
            if result == CliPlayer::from("".to_owned()) {
                break None;
            } else if possible_targets.contains(&&result) {
                break Some(result);
            }
            self.print_secret("no such player");
        }
    }

    fn choose_investigation_target(&self, possible_targets: Vec<&CliPlayer>) -> Option<CliPlayer> {
        loop {
            let result = CliPlayer::from(self.input_secret("player to investigate"));
            if result == CliPlayer::from("".to_owned()) {
                break None;
            } else if possible_targets.contains(&&result) {
                break Some(result);
            }
            self.print_secret("no such player");
        }
    }

    fn recv_investigation(&self, faction: Faction) {
        self.print_secret(&format!("investigation result: {}", faction)[..]);
    }

    fn choose_werewolf_kill_target(&self, possible_targets: Vec<&CliPlayer>) -> CliPlayer {
        loop {
            let result = CliPlayer::from(self.input_secret("player to werewolf-kill"));
            if possible_targets.contains(&&result) {
                break result;
            }
            self.print_secret("no such player");
        }
    }

    fn recv_exile(&self, reason: &str) {
        self.print_secret(&format!("you have been exiled for {}", reason)[..]);
    }
}

impl fmt::Display for CliPlayer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.name.fmt(f)
    }
}
