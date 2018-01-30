use std::fmt;

use util;
use game::{Faction, Role};
use handler::Handler;
use player::Player;

/// A game handler which uses the command line.
pub struct CliHandler;

impl<P: Player + From<String> + fmt::Display> Handler<P> for CliHandler {
    fn announce_deaths<I: IntoIterator<Item = (P, Role)>>(&mut self, deaths: I) {
        for (player, role) in deaths {
            println!("[ ** ] {} died and was {}", player, role);
        }
    }

    fn announce_probability_table<I: IntoIterator<Item = Result<(f64, f64, f64), Faction>>>(&mut self, probability_table: I) {
        for (player_idx, probabilities) in probability_table.into_iter().enumerate() {
            match probabilities {
                Ok((village_ratio, werewolves_ratio, dead_ratio)) => {
                    println!("[ ** ] {}: {}% village, {}% werewolf, {}% dead", player_idx, (village_ratio * 100.0).round() as u8, (werewolves_ratio * 100.0).round() as u8, (dead_ratio * 100.0).round() as u8);
                }
                Err(faction) => {
                    println!("[ ** ] {}: dead (was {})", player_idx, faction);
                }
            }
        }
    }

    fn cannot_lynch(&mut self, _: P) {
        println!("[ !! ] no such player to lynch");
    }

    fn choose_lynch_target(&mut self) -> Option<P> {
        let name = util::input("town lynch target");
        if name == "no lynch" {
            None
        } else {
            Some(P::from(name))
        }
    }
}
