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
}

impl Player for CliPlayer {
    fn name(&self) -> &str {
        &self.name
    }
}
