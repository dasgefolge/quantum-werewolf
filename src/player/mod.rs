//! Contains the `Player` trait, which is what the game uses to talk to players, and some implementations.

mod cli;

use std::fmt;
use crate::game::Faction;
pub use self::cli::CliPlayer;

/// The game uses this trait to talk to players. Implementing types perform all game actions.
pub trait Player: fmt::Debug + ::std::marker::Sized {
    /// Notifies the player that they have received a secret player ID.
    fn recv_id(&self, player_id: usize);

    /// Called when the player should heal a player. Should return the name of the player to heal.
    ///
    /// Returning the name of a dead player or a name not in the game is treated the same as not healing anyone.
    fn choose_heal_target(&self, possible_targets: Vec<&Self>) -> Option<Self>;

    /// Called when the player should investigate another player. Should return the name of the investigated player.
    ///
    /// Returning the name of a dead player, one's own name, or a name not in the game is treated the same as not investigating anyone.
    fn choose_investigation_target(&self, possible_targets: Vec<&Self>) -> Option<Self>;

    /// Notifies the player of the result of an investigation.
    fn recv_investigation(&self, faction: Faction);

    /// Called when the player should kill another player as the dominant werewolf. Should return the name of the attacked player.
    ///
    /// An illegal choice will exile the player.
    fn choose_werewolf_kill_target(&self, possible_targets: Vec<&Self>) -> Self;

    /// Called when the player is exiled from the game.
    fn recv_exile(&self, reason: &str);
}
