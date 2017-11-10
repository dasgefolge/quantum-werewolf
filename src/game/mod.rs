//! Core game logic.

pub mod state;
mod types;

use std::fmt;
use std::collections::HashSet;
use std::hash::Hash;

use util;
use player::Player;
pub use self::types::*;

/// Generate a basic role distribution for the signed-up players, and moderate a game of Quantum Werewolf.
///
/// The number of werewolves will be the 0.4 times the number of players, rounded down. There will also be one detective.
///
/// Returns the winners of the game.
pub fn run<P: Eq + Hash + Clone + Player + fmt::Display + From<String>>(game_state: state::Signups<P>) -> Result<HashSet<P>, state::StartGameError> {
    let num_ww = game_state.num_players() * 2 / 5;
    let mut roles = (0..num_ww).map(|i| Role::Werewolf(i)).collect::<Vec<_>>();
    roles.push(Role::Detective);
    run_with_roles(game_state, roles)
}

/// Moderate a game of Quantum Werewolf with the given players and roles.
///
/// If fewer roles than players are given, a number of Villagers equal to the difference will be added.
///
/// Returns the winners of the game.
pub fn run_with_roles<P: Eq + Hash + Clone + Player + fmt::Display + From<String>>(game_state: state::Signups<P>, roles: Vec<Role>) -> Result<HashSet<P>, state::StartGameError> {
    let mut game_state = game_state.start(roles)?;
    let mut alive = game_state.alive().expect("failed to get list of living players").into_iter().cloned().collect::<HashSet<_>>();
    // assign secret player IDs
    for (i, player) in game_state.secret_ids().expect("failed to get secred player IDs").into_iter().enumerate() {
        player.recv_id(i);
    }
    Ok(loop {
        if let Some(new_alive) = game_state.alive() {
            let new_alive = new_alive.into_iter().cloned().collect();
            for name in &alive - &new_alive {
                if let Some(role) = game_state.role(&name) {
                    //TODO send to players
                    println!("[ ** ] {} died and was {}", name, role);
                }
            }
            alive = new_alive;
        }
        game_state = match game_state {
            state::State::Signups(_) => unreachable!(),
            state::State::Night(night) => {
                night.resolve(
                    |p, targets| p.choose_heal_target(targets),
                    |p, targets| p.choose_investigation_target(targets),
                    |p, targets| p.choose_werewolf_kill_target(targets)
                )
            }
            state::State::Day(day) => {
                // send night action results
                for (player, result) in day.night_action_results() {
                    match result {
                        NightActionResult::Investigation(faction) => { player.recv_investigation(faction); }
                    }
                }
                // announce probability table
                for (player_idx, probabilities) in day.multiverse.probability_table().into_iter().enumerate() {
                    match probabilities {
                        Ok((village_ratio, werewolves_ratio, dead_ratio)) => {
                            println!("[ ** ] {}: {}% village, {}% werewolf, {}% dead", player_idx, (village_ratio * 100.0).round() as u8, (werewolves_ratio * 100.0).round() as u8, (dead_ratio * 100.0).round() as u8);
                        }
                        Err(faction) => {
                            println!("[ ** ] {}: dead (was {})", player_idx, faction);
                        }
                    }
                }
                //TODO send to players
                //TODO nominations
                //TODO vote
                loop {
                    let name = util::input("town lynch target");
                    if name == "no lynch" {
                        break day.no_lynch();
                    } else {
                        let target = P::from(name);
                        if day.can_lynch(&target) {
                            break day.lynch(target);
                        }
                        println!("[ !! ] no such player to lynch");
                    }
                }
            }
            state::State::Complete(state::Complete { winners }) => { break winners; }
        };
    })
}
