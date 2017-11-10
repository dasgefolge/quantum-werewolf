//! Data types used in game state representation.

use std::{fmt, iter, mem, slice, vec};
use std::collections::HashMap;
use std::str::FromStr;

use rand::thread_rng;

use util::QwwIteratorExt;

/// The faction (also called party) of a player determines their goal. It is usually derived from the role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Faction {
    /// The player wants to eliminate the village.
    Werewolves,
    /// The player wants to eliminate all threats to the village.
    Village
}

impl Faction {
    /// Checks whether a faction's win condition has been met in the given universe.
    pub fn wincon(&self, universe: &Universe) -> bool {
        match *self {
            Faction::Werewolves => !universe.factions.iter()
                .zip(universe.alive.iter())
                .any(|(&faction, &alive)| faction == Faction::Village && alive),
            Faction::Village => {
                let villager_alive = universe.factions.iter()
                    .zip(universe.alive.iter())
                    .any(|(&faction, &alive)| faction == Faction::Village && alive);
                // if introducing additional factions considered threats to the village, update this
                let threat_alive = universe.factions.iter()
                    .zip(universe.alive.iter())
                    .any(|(&faction, &alive)| faction == Faction::Werewolves && alive);
                villager_alive && !threat_alive
            }
        }
    }
}

impl fmt::Display for Faction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Faction::Werewolves => write!(f, "werewolves"),
            Faction::Village => write!(f, "village")
        }
    }
}

/// Contains the information sent to a player as the result of a night action.
#[derive(Debug, Clone, Copy)]
pub enum NightActionResult {
    /// An investigation result, for example for a detective.
    Investigation(Faction)
}

/// A Werewolf player role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// A detective, part of the village. Investigates a player each night, learning their faction.
    Detective,
    /// A healer, part of the village. Heals a player each night, making them immortal for the night. May not heal the same player two nights in a row.
    Healer,
    /// A regular villager with no special abilities.
    Villager,
    /// A werewolf. Kills a player each night if no werewolf with a *lower* rank is alive.
    Werewolf(usize)
}

impl Role {
    fn default_faction(&self) -> Faction {
        match *self {
            Role::Detective | Role::Healer | Role::Villager => Faction::Village,
            Role::Werewolf(_) => Faction::Werewolves
        }
    }
}

impl FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Role, ()> {
        match &s.to_lowercase()[..] {
            "detective" => Ok(Role::Detective),
            "healer" => Ok(Role::Healer),
            "villager" => Ok(Role::Villager),
            "werewolf" => Ok(Role::Werewolf(0)),
            _ => Err(())
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Role::Detective => write!(f, "detective"),
            Role::Healer => write!(f, "healer"),
            Role::Villager => write!(f, "villager"),
            Role::Werewolf(i) => write!(f, "werewolf {}", i)
        }
    }
}

/// A universe represents one of the possible quantum states in a game of Quantum Werewolf. It contains information such as the distribution of roles, and which players are still alive.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Universe {
    pub(crate) alive: Vec<bool>,
    pub(crate) roles: Vec<Role>,
    pub(crate) factions: Vec<Faction>,
    pub(crate) heals: Vec<usize>, // this should be a set, but HashSet isn't hashable
    pub(crate) kills: Vec<usize> // this should be a set, but HashSet isn't hashable
}

impl Universe {
    /// Checks if the game has ended.
    fn game_over(&self, night: bool) -> bool {
        self.alive.iter().all(|alive| !alive) ||
        (!night && self.alive.iter().filter(|&&alive| alive).count() < 2) ||
        (!night && self.factions.iter().any(|faction| faction.wincon(self)))
    }

    /// Utility method to properly handle killing a player depending on day/night, healed status, etc.
    pub fn kill(&mut self, player_idx: usize, night: bool) {
        if night {
            if !self.heals.contains(&player_idx) {
                self.kills.push(player_idx);
            }
        } else {
            self.alive[player_idx] = false;
        }
    }
}

impl From<Vec<Role>> for Universe {
    fn from(roles: Vec<Role>) -> Universe {
        Universe {
            alive: vec![true; roles.len()],
            factions: roles.iter().map(Role::default_faction).collect(),
            roles: roles,
            heals: Vec::default(),
            kills: Vec::default()
        }
    }
}

/// A collection of universes, with several convenience methods.
pub struct Multiverse(Vec<Universe>);

impl Multiverse {
    /// Constructs and returns a new multiverse with all possible role distributions.
    ///
    /// If fewer roles than players are given, the remaining role slots will be populated with villagers.
    pub fn new(roles: Vec<Role>, num_players: usize) -> Multiverse {
        let mut permutations = vec![vec![Role::Villager; num_players]];
        for role in roles {
            permutations = permutations.into_iter().flat_map(|perm| {
                (0..perm.len()).filter_map(|i| {
                    if perm[i] == Role::Villager {
                        let mut new_perm = perm.clone();
                        new_perm[i] = role;
                        Some(new_perm)
                    } else {
                        None
                    }
                }).collect::<Vec<_>>()
            }).collect();
        }
        Multiverse(permutations.into_iter().map(Universe::from).collect())
    }

    /// Returns the set of players which are still alive in at least one possible universe.
    pub fn alive(&self) -> Vec<usize> {
        (0..self.num_players()).filter(|&idx| {
            self.0.iter().any(|universe| universe.alive[idx])
        }).collect()
    }

    /// Determines a single role for each dead player and removes all universes where that player doesn't have that role.
    pub fn collapse_roles(&mut self) {
        let mut start_size = self.0.len();
        let mut collapsed_roles = HashMap::<usize, Role>::default();
        loop {
            {
                let collapse_universe = self.iter().rand(&mut thread_rng()).expect("paradox created while collapsing roles");
                for player_idx in 0..self.num_players() {
                    if !self.alive().contains(&player_idx) {
                        collapsed_roles.insert(player_idx, collapse_universe.roles[player_idx]);
                    }
                }
            }
            let multiverse = mem::replace(&mut self.0, Vec::default());
            self.0 = multiverse
                .into_iter()
                .filter(|universe| collapsed_roles.iter().all(|(&id, &role)| universe.roles[id] == role))
                .collect();
            if self.0.len() == start_size {
                break;
            } else {
                start_size = self.0.len();
            }
        }
    }

    /// Returns the faction of the given player, if that faction is unambiguous.
    pub fn faction(&self, player_idx: usize) -> Option<Faction> {
        let faction = self.0[0].factions[player_idx];
        if self.0[1..].iter().all(|universe| universe.factions[player_idx] == faction) {
            Some(faction)
        } else {
            None
        }
    }

    /// Checks if the game has ended in all universes.
    pub fn game_over(&self, night: bool) -> bool {
        self.iter().all(|universe| universe.game_over(night))
    }

    /// Iterates over all universes in no particular order.
    pub fn into_iter(self) -> vec::IntoIter<Universe> {
        self.0.into_iter()
    }

    /// Iterates over all universes in no particular order.
    pub fn iter(&self) -> slice::Iter<Universe> {
        self.0.iter()
    }

    /// Iterates over all universes in no particular order.
    pub fn iter_mut(&mut self) -> slice::IterMut<Universe> {
        self.0.iter_mut()
    }

    /// The total number of players for which this multiverse was created.
    pub fn num_players(&self) -> usize {
        self.0[0].alive.len()
    }

    /// Produces the anonymized probability table shown to players at the start of the day.
    pub fn probability_table(&self) -> Vec<Result<(f64, f64, f64), Faction>> {
        (0..self.num_players()).into_iter().map(|player_idx| {
            if self.alive().contains(&player_idx) {
                let village_universes = self.iter()
                    .filter(|universe| universe.factions[player_idx] == Faction::Village)
                    .count();
                let village_ratio = (village_universes as f64) / (self.0.len() as f64);
                let werewolves_universes = self.iter()
                    .filter(|universe| universe.factions[player_idx] == Faction::Werewolves)
                    .count();
                let werewolves_ratio = (werewolves_universes as f64) / (self.0.len() as f64);
                let dead_universes = self.iter()
                    .filter(|universe| !universe.alive[player_idx])
                    .count();
                let dead_ratio = (dead_universes as f64) / (self.0.len() as f64);
                Ok((village_ratio, werewolves_ratio, dead_ratio))
            } else {
                Err(self.faction(player_idx).expect("player is dead but does not have a determined faction"))
            }
        }).collect()
    }

    /// Returns the role of the given player, if that role is unambiguous.
    pub fn role(&self, player_idx: usize) -> Option<Role> {
        let role = self.0[0].roles[player_idx];
        if self.0[1..].iter().all(|universe| universe.roles[player_idx] == role) {
            Some(role)
        } else {
            None
        }
    }

    /// Whether or not there is at least one player who may be alive and may have this role.
    ///
    /// Note that this may return true even if this role is dead in all universes, namely if there are multiple players who may have that role.
    pub fn role_alive(&self, role: Role) -> bool {
        self.iter().any(|universe| {
            universe.roles.iter().enumerate().any(|(player_idx, &iter_role)|
                role == iter_role && (
                    universe.alive[player_idx] ||
                    self.role(player_idx).is_none()
                )
            )
        })
    }
}

impl iter::FromIterator<Universe> for Multiverse {
    fn from_iter<I: IntoIterator<Item=Universe>>(iter: I) -> Multiverse {
        Multiverse(iter.into_iter().collect())
    }
}
