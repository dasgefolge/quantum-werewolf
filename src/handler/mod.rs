//! Contains the `Handler` trait, which is what the game uses to broadcast public game messages, and some implementations.

mod cli;

use std::collections::HashSet;

use game::{Faction, Role};
use player::Player;
pub use self::cli::CliHandler;

/// The game uses this trait to broadcast public game messages.
pub trait Handler<P: Player> {
    /// Called when one or more players die. Includes a copy of the player and the flipped role.
    fn announce_deaths<I: IntoIterator<Item = (P, Role)>>(&mut self, _: I) {}

    /// Called at the start of the day to announce the probability table.
    ///
    /// The iterable can be enumerated to generate the secret IDs corresponding to the probabilities.
    fn announce_probability_table<I: IntoIterator<Item = Result<(f64, f64, f64), Faction>>>(&mut self, _: I) {}

    /// Called if an invalid player has been chosen as a lynch target.
    ///
    /// A call of this method is followed up by another `choose_lynch_target` call to restart the discussion.
    fn cannot_lynch(&mut self, _: P) {}

    /// Called at the start of the day determine the lynch target.
    ///
    /// Implementations should run the town discussion, implementing any appropriate discussion system, and return the lynched player.
    ///
    /// Returning `None` stands for a no-lynch decision.
    fn choose_lynch_target(&mut self, _: HashSet<&P>) -> Option<P>;
}
