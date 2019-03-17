//! Core game logic.

pub mod state;
mod types;

use std::{
    collections::HashSet,
    hash::Hash
};
use crate::{
    handler::Handler,
    player::Player
};
pub use self::types::*;

/// Generate a basic role distribution for the signed-up players, and moderate a game of Quantum Werewolf.
///
/// The number of werewolves will be the 0.4 times the number of players, rounded down. There will also be one detective.
///
/// Returns the winners of the game.
pub fn run<P: Eq + Hash + Clone + Player, H: Handler<P>>(handler: H, game_state: state::Signups<P>) -> Result<HashSet<P>, state::StartGameError> {
    let num_ww = game_state.num_players() * 2 / 5;
    let mut roles = (0..num_ww).map(|i| Role::Werewolf(i)).collect::<Vec<_>>();
    roles.push(Role::Detective);
    run_with_roles(handler, game_state, roles)
}

/// Moderate a game of Quantum Werewolf with the given players and roles.
///
/// If fewer roles than players are given, a number of Villagers equal to the difference will be added.
///
/// Returns the winners of the game.
pub fn run_with_roles<P: Eq + Hash + Clone + Player, H: Handler<P>>(mut handler: H, game_state: state::Signups<P>, roles: Vec<Role>) -> Result<HashSet<P>, state::StartGameError> {
    let mut game_state = game_state.start(roles)?;
    let mut alive = game_state.alive().expect("failed to get list of living players").into_iter().cloned().collect::<HashSet<_>>();
    // assign secret player IDs
    for (i, player) in game_state.secret_ids().expect("failed to get secred player IDs").into_iter().enumerate() {
        player.recv_id(i);
    }
    Ok(loop {
        if let Some(new_alive) = game_state.alive() {
            let new_alive = new_alive.into_iter().cloned().collect();
            handler.announce_deaths((&alive - &new_alive).into_iter()
                .filter_map(|player| game_state.role(&player).map(|role| (player, role)))
            );
            alive = new_alive;
        }
        game_state = match game_state {
            state::State::Signups(_) => unreachable!(),
            state::State::Night(night) => {
                night.resolve_tar(
                    |p, targets| p.choose_heal_target(targets),
                    |p, targets| p.choose_investigation_target(targets),
                    |p, targets| p.choose_werewolf_kill_target(targets)
                )
            }
            state::State::Day(day) => {
                // send night action results
                for (player, result) in day.night_action_results() {
                    match result {
                        NightActionResult::Investigation(target, faction) => { player.recv_investigation(target, faction); }
                    }
                }
                // announce probability table
                handler.announce_probability_table(day.probability_table());
                // vote
                loop {
                    if let Some(target) = handler.choose_lynch_target(day.alive()) {
                        if day.can_lynch(&target) {
                            break day.lynch(target);
                        }
                        handler.cannot_lynch(target);
                    } else {
                        break day.no_lynch();
                    }
                }
            }
            state::State::Complete(state::Complete { winners }) => { break winners; }
        };
    })
}
