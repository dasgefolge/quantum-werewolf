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
}
