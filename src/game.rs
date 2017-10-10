//! Core game logic.

use std::{fmt, mem};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use rand::{Rng, thread_rng};

use player::Player;
use util::{QwwIteratorExt, input};

/// The possible errors returned by `Game::new`.
#[derive(Debug)]
pub enum NewGameError {
    /// There are less than three players.
    NotEnoughPlayers,
    /// Multiple players have the same name.
    NameCollision(String),
    /// More roles than there are players have been specified.
    TooManyRoles
}

/// The party of a player determines their goal. It is usually derived from the role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Party {
    /// The player wants to eliminate the village.
    Werewolves,
    /// The player wants to eliminate all threats to the village.
    Village
}

impl fmt::Display for Party {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Party::Werewolves => write!(f, "werewolves"),
            Party::Village => write!(f, "village")
        }
    }
}

/// A Werewolf player role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// A detective, part of the village. Investigates a player each night, learning their party.
    Detective,
    /// A healer, part of the village. Heals a player each night, making them immortal for the night. May not heal the same player two nights in a row.
    Healer,
    /// A regular villager with no special abilities.
    Villager,
    /// A werewolf. Kills a player each night if no werewolf with a lower rank is alive.
    Werewolf(usize)
}

impl Role {
    fn default_party(&self) -> Party {
        match *self {
            Role::Detective | Role::Healer | Role::Villager => Party::Village,
            Role::Werewolf(_) => Party::Werewolves
        }
    }
}

impl FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Role, ()> {
        match s {
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

#[derive(Debug, PartialEq, Eq, Hash)]
struct Universe {
    alive: Vec<bool>,
    roles: Vec<Role>,
    parties: Vec<Party>,
    heals: Vec<usize>, // this should be a set, but HashSet isn't hashable
    kills: Vec<usize> // this should be a set, but HashSet isn't hashable
}

impl Universe {
    fn kill(&mut self, player_id: usize, night: bool) {
        if night {
            if !self.heals.contains(&player_id) {
                self.kills.push(player_id);
            }
        } else {
            self.alive[player_id] = false;
        }
    }
}

impl From<Vec<Role>> for Universe {
    fn from(roles: Vec<Role>) -> Universe {
        Universe {
            alive: vec![true; roles.len()],
            parties: roles.iter().map(Role::default_party).collect(),
            roles: roles,
            heals: Vec::default(),
            kills: Vec::default()
        }
    }
}

/// This represents the state of a game.
#[derive(Debug)]
pub struct Game {
    players: HashMap<String, Box<Player>>,
    player_ids: HashMap<String, usize>,
    multiverse: HashSet<Universe>
}

impl Game {
    /// Creates a new game from a list of players. A set of roles may optionally be given; if omitted, the only roles will be werewolves, villagers, and a detective.
    ///
    /// # Errors
    ///
    /// Will return an error if no game can be created with the given player list. See `NewGameError` for details.
    pub fn new(players: Vec<Box<Player>>, roles: Option<Vec<Role>>) -> Result<Game, NewGameError> {
        // validate player list
        if players.len() < 3 {
            return Err(NewGameError::NotEnoughPlayers);
        }
        let mut player_map = HashMap::default();
        for player in players {
            let name = player.name().to_owned();
            if player_map.contains_key(&name) {
                return Err(NewGameError::NameCollision(name));
            }
            player_map.insert(name, player);
        }
        // assign secret player IDs
        let mut player_ids = HashMap::default();
        for (i, (name, _)) in player_map.iter().enumerate() {
            player_ids.insert(name.to_owned(), i);
        }
        {
            let mut shuffled_ids = player_ids.iter().collect::<Vec<_>>();
            thread_rng().shuffle(&mut shuffled_ids);
            for (player_name, &i) in shuffled_ids {
                player_map.get(player_name).expect("failed to distribute player IDs").recv_id(i);
            }
        }
        // generate multiverse
        let roles = if let Some(roles) = roles {
            if roles.len() > player_map.len() {
                return Err(NewGameError::TooManyRoles);
            }
            roles.into_iter()
                .filter(|&role| role != Role::Villager)
                .fold((0, Vec::default()), |(mut num_ww, mut roles), role| {
                    if let Role::Werewolf(_) = role {
                        roles.push(Role::Werewolf(num_ww));
                        num_ww += 1;
                    } else {
                        roles.push(role);
                    }
                    (num_ww, roles)
                }).1
        } else {
            let num_ww = player_map.len() / 3;
            let mut result = (0..num_ww).map(|i| Role::Werewolf(i)).collect::<Vec<_>>();
            result.push(Role::Detective);
            result
        };
        let mut permutations = vec![vec![Role::Villager; player_map.len()]];
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
        let multiverse = permutations.into_iter().map(Universe::from).collect();
        Ok(Game {
            players: player_map,
            player_ids: player_ids,
            multiverse: multiverse
        })
    }

    fn collapse_roles(&mut self) {
        let mut start_size = self.multiverse.len();
        let mut collapsed_roles = HashMap::<usize, Role>::default();
        loop {
            for name in self.player_names() {
                if let Some(&id) = self.player_ids.get(&name) {
                    if self.multiverse.iter().all(|universe| !universe.alive[id]) {
                        collapsed_roles.insert(id, self.multiverse.iter().rand(&mut thread_rng()).expect(&format!("failed to collapse role for {}", name)).roles[id]);
                    }
                }
            }
            let multiverse = mem::replace(&mut self.multiverse, HashSet::default());
            self.multiverse = multiverse
                .into_iter()
                .filter(|universe| collapsed_roles.iter().all(|(&id, &role)| universe.roles[id] == role))
                .collect();
            if self.multiverse.len() == start_size {
                break;
            } else if self.multiverse.is_empty() {
                panic!("paradox created while collapsing roles");
            } else {
                start_size = self.multiverse.len();
            }
        }
    }

    //fn exile(&mut self, name: &str, reason: &str) {
    //    if let Some(player) = self.players.remove(name) {
    //        player.recv_exile(reason);
    //        self.player_ids.remove(name);
    //    }
    //}

    fn maybe_alive(&self, role: Role) -> bool {
        if self.multiverse.iter().all(|universe| !universe.roles.contains(&role)) {
            // role is not in the setup in the first place
            return false;
        }
        // check not only if the role is dead in all universes, but also if it's the same player with the role (to avoid giving away extra information)
        !self.player_names().into_iter().any(|name| {
            let &id = self.player_ids.get(&name).expect("player ID not found");
            self.multiverse.iter().all(|universe| !universe.alive[id] && universe.roles[id] == role)
        })
    }

    fn player_names(&self) -> Vec<String> {
        let mut result = self.players.keys()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        thread_rng().shuffle(&mut result);
        result
    }

    //fn players(&self) -> Vec<(&str, &(Player + 'static))> {
    //    let mut result = self.players.iter()
    //        .map(|(k, v)| (&k[..], &**v))
    //        .collect::<Vec<_>>();
    //        thread_rng().shuffle(&mut result);
    //    result
    //}

    /// Runs the entire game and returns the names of the winners.
    pub fn run(mut self) -> HashSet<String> {
        // night/day loop
        loop {
            // night
            self.multiverse = self.multiverse.into_iter()
                .map(|mut universe| {
                    universe.heals = Vec::default();
                    universe.kills = Vec::default();
                    universe
                })
                .collect();
            let alive_at_night_start = self.player_ids
                .iter()
                .map(|(_, &id)| id)
                .filter(|&id| self.multiverse.iter().any(|universe| universe.alive[id]))
                .collect::<HashSet<_>>();
            // healer actions
            if self.maybe_alive(Role::Healer) {
                for name in self.player_names() {
                    let player = self.players.get(&name).expect("player name not found");
                    let &id = self.player_ids.get(&name).expect("player ID not found");
                    if self.multiverse.iter().any(|universe| universe.alive[id]) {
                        if let Some(target) = player.choose_heal_target(self.player_names()) {
                            if let Some(&target_id) = self.player_ids.get(&target) {
                                // heal target in all gamestates where player is healer
                                self.multiverse = self.multiverse.into_iter()
                                    .map(|mut universe| {
                                        let can_heal = universe.roles[id] == Role::Healer &&
                                        universe.alive[id] &&
                                        universe.alive[target_id]; //TODO forbid healing the same player 2 nights in a row
                                        if can_heal {
                                            universe.heals.push(target_id);
                                        }
                                        universe
                                    })
                                    .collect();
                            }
                        }
                    }
                }
            }
            // detective investigations
            if self.maybe_alive(Role::Detective) {
                for name in self.player_names() {
                    let player = self.players.get(&name).expect("player name not found");
                    let &id = self.player_ids.get(&name).expect("player ID not found");
                    if self.multiverse.iter().any(|universe| universe.alive[id]) {
                        if let Some(target) = player.choose_investigation_target(self.player_names()) {
                            if let Some(&target_id) = self.player_ids.get(&target) {
                                // investigate player in all gamestates where player is detective
                                let investigated_party = if let Some(investigation_universe) = self.multiverse.iter()
                                    .filter(|universe| universe.roles[id] == Role::Detective) // player must be detective,
                                    .filter(|universe| universe.alive[id]) // and detective must be alive
                                    .rand(&mut thread_rng())
                                {
                                    investigation_universe.parties[target_id]
                                } else {
                                    continue;
                                };
                                player.recv_investigation(&target, investigated_party);
                                self.multiverse = self.multiverse.into_iter()
                                    .filter(|universe| !(
                                        universe.roles[id] == Role::Detective &&
                                        universe.alive[id] &&
                                        universe.parties[target_id] != investigated_party
                                    ))
                                    .collect();
                            }
                        }
                    }
                }
            }
            // werewolf kills
            for name in self.player_names() {
                let player = self.players.get(&name).expect("player name not found");
                let &id = self.player_ids.get(&name).expect("player ID not found");
                if self.multiverse.iter().any(|universe| universe.alive[id]) {
                    let target = player.choose_werewolf_kill_target(self.player_names());
                    if let Some(&target_id) = self.player_ids.get(&target) {
                        // kill target in all gamestates where player is first-ranking werewolf alive
                        self.multiverse = self.multiverse.into_iter()
                            .map(|mut universe| {
                                let can_kill = if let Role::Werewolf(werewolf_rank) = universe.roles[id] {
                                    universe.roles
                                        .iter()
                                        .enumerate()
                                        .all(|(i, role)| if let &Role::Werewolf(cmp_rank) = role {
                                            cmp_rank >= werewolf_rank || !universe.alive[i]
                                        } else {
                                            true
                                        })
                                } else {
                                    false
                                } &&
                                universe.alive[id] &&
                                universe.alive[target_id];
                                if can_kill {
                                    universe.kill(target_id, true);
                                }
                                universe
                            })
                            .collect();
                    } else {
                        //TODO exile werewolf
                    }
                }
            }
            // morning
            self.multiverse = self.multiverse.into_iter()
                .map(|mut universe| {
                    for &player_id in &universe.kills {
                        universe.alive[player_id] = false;
                    }
                    universe
                })
                .collect();
            self.collapse_roles();
            // announce night deaths
            let alive_at_night_end = self.player_ids
                .iter()
                .map(|(_, &id)| id)
                .filter(|&id| self.multiverse.iter().any(|universe| universe.alive[id]))
                .collect::<HashSet<_>>();
            for name in self.player_names() {
                let &id = self.player_ids.get(&name).expect("player ID not found");
                if alive_at_night_start.contains(&id) && !alive_at_night_end.contains(&id) {
                    if let Some(sample_universe) = self.multiverse.iter().next() {
                        //TODO send to players
                        println!("[ ** ] {} died and was {}", name, sample_universe.roles[id]);
                    }
                }
            }
            //TODO check for game-ending conditions
            // announce probability table
            for (_, &id) in self.player_ids.iter() {
                if self.multiverse.iter().any(|universe| universe.alive[id]) {
                    let village_universes = self.multiverse
                        .iter()
                        .filter(|universe| universe.parties[id] == Party::Village)
                        .count();
                    let village_ratio = (village_universes as f64) / (self.multiverse.len() as f64);
                    let werewolves_universes = self.multiverse
                        .iter()
                        .filter(|universe| universe.parties[id] == Party::Werewolves)
                        .count();
                    let werewolves_ratio = (werewolves_universes as f64) / (self.multiverse.len() as f64);
                    let dead_universes = self.multiverse
                        .iter()
                        .filter(|universe| !universe.alive[id])
                        .count();
                    let dead_ratio = (dead_universes as f64) / (self.multiverse.len() as f64);
                    println!("[ ** ] {}: {}% village, {}% werewolf, {}% dead", id, (village_ratio * 100.0).round() as u8, (werewolves_ratio * 100.0).round() as u8, (dead_ratio * 100.0).round() as u8);
                } else {
                    let sample_universe = self.multiverse.iter().next().expect("multiverse is empty");
                    println!("[ ** ] {}: dead (was {})", id, sample_universe.parties[id]);
                }
            }
            //TODO send to players
            // day
            let alive_at_day_start = self.player_ids
                .iter()
                .map(|(_, &id)| id)
                .filter(|&id| self.multiverse.iter().any(|universe| universe.alive[id]))
                .collect::<HashSet<_>>();
            //TODO nominations
            //TODO vote
            let lynch_name = input("town lynch target");
            if lynch_name != "no lynch" {
                let &lynch_id = self.player_ids.get(&lynch_name).expect("failed to get town lynch target");
                // evening
                // eliminate impossible gamestates (where the player to be killed by the vote is already dead), then kill voted player
                self.multiverse = self.multiverse.into_iter()
                    .filter(|universe| universe.alive[lynch_id])
                    .map(|mut universe| {
                        universe.kill(lynch_id, false);
                        universe
                    })
                    .collect();
                self.collapse_roles();
                // announce day deaths
                let alive_at_day_end = self.player_ids
                    .iter()
                    .map(|(_, &id)| id)
                    .filter(|&id| self.multiverse.iter().any(|universe| universe.alive[id]))
                    .collect::<HashSet<_>>();
                for name in self.player_names() {
                    if let Some(&id) = self.player_ids.get(&name) {
                        if alive_at_day_start.contains(&id) && !alive_at_day_end.contains(&id) {
                            if let Some(sample_universe) = self.multiverse.iter().next() {
                                //TODO send to players
                                println!("[ ** ] {} died and was {}", name, sample_universe.roles[id]);
                            }
                        }
                    }
                }
            }
            //TODO check for game-ending conditions
        }
    }
}
