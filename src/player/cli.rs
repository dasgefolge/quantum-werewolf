use std::io::prelude::*;
use std::io::{stdin, stdout};

use game::Party;
use player::Player;

/// A player who sends game actions via the command line.
#[derive(Debug)]
pub struct CliPlayer {
    name: String
}

impl CliPlayer {
    /// Creates a new CLI player with the given player name.
    pub fn new(name: String) -> CliPlayer {
        CliPlayer {
            name: name
        }
    }

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

impl Player for CliPlayer {
    fn name(&self) -> &str {
        &self.name
    }

    fn recv_id(&self, player_id: usize) {
        self.print_secret(&format!("your secret player ID is {}", player_id)[..]);
    }

    fn choose_heal_target(&self) -> Option<String> {
        let result = self.input_secret("player to heal");
        if result == "" {
            None
        } else {
            Some(result)
        }
    }

    fn choose_investigation_target(&self) -> Option<String> {
        let result = self.input_secret("player to investigate");
        if result == "" {
            None
        } else {
            Some(result)
        }
    }

    fn recv_investigation(&self, player_name: &str, party: Party) {
        self.print_secret(&format!("{} investigated as {}", player_name, party)[..]);
    }

    fn choose_werewolf_kill_target(&self) -> String {
        self.input_secret("player to werewolf-kill")
    }

    fn recv_exile(&self, reason: &str) {
        self.print_secret(&format!("you have been exiled for {}", reason)[..]);
    }
}
