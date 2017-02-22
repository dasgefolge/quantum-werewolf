//! Contains the `Player` trait, which is what the game uses to talk to players, and some implementations.

mod cli;

use std::fmt;

pub use self::cli::CliPlayer;

/// The game uses this trait to talk to players. Implementing types perform all game actions.
pub trait Player: fmt::Debug {
    /// Returns the name of the player. Must stay the same throughout a game.
    fn name(&self) -> &str;

    /// Notifies the player that they have received a secret player ID.
    fn recv_id(&self, usize);
}
