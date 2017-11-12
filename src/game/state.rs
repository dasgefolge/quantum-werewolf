//! Game state representation.

use std::fmt;
use std::collections::HashSet;
use std::hash::Hash;

use rand::{Rng, thread_rng};

use game::{Multiverse, NightActionResult, Role};
use util::QwwIteratorExt;

/// The minimum number of players required to start a game.
pub const MIN_PLAYERS: usize = 3;

/// This enum represents the state of the game. Each variant contains relevant methods to observe or progress the game state, refer to their documentation for details.
///
/// The type parameter `P` is used for player identifiers.
pub enum State<P: Eq + Hash> {
    /// A game which has not been started. The moderator may sign up players, or start the game.
    Signups(Signups<P>),
    /// A running game which is currently in night time, waiting for the players' night actions.
    Night(Night<P>),
    /// A running game which is currently in day time, waiting for the result of the lynch vote.
    Day(Day<P>),
    /// A completed game.
    Complete(Complete<P>)
}

impl<P: Eq + Hash> State<P> {
    /// If the game is onging, returns the set of players which are still alive in at least one possible universe.
    pub fn alive(&self) -> Option<HashSet<&P>> {
        match *self {
            State::Signups(_) => None,
            State::Night(ref night) => Some((night.secret_ids(), night.multiverse.alive())),
            State::Day(ref day) => Some((day.secret_ids(), day.multiverse.alive())),
            State::Complete(_) => None
        }.map(|(ids, idxs)| idxs.into_iter().map(|idx| &ids[idx]).collect())
    }

    /// Returns the number of players that are in this game
    ///
    /// For `Signups`, this is the number of players that have been signed up so far. For `Complete`, it's the number of winners.
    pub fn num_players(&self) -> usize {
        match *self {
            State::Signups(ref signups) => signups.num_players(),
            State::Night(ref night) => night.secret_ids().len(),
            State::Day(ref day) => day.secret_ids().len(),
            State::Complete(Complete { ref winners }) => winners.len()
        }
    }

    /// Returns the role of the given player, if that role is unambiguous.
    pub fn role(&self, player: &P) -> Option<Role> {
        match *self {
            State::Signups(_) => None,
            State::Night(ref night) => night.multiverse.role(night.secret_ids.iter().position(|iter_player| player == iter_player).expect("no such player")),
            State::Day(ref day) => day.multiverse.role(day.secret_ids.iter().position(|iter_player| player == iter_player).expect("no such player")),
            State::Complete(_) => None
        }
    }

    /// If the game is ongoing, returns the player list, sorted by secret player ID.
    pub fn secret_ids(&self) -> Option<&[P]> {
        match *self {
            State::Signups(_) => None,
            State::Night(ref night) => Some(night.secret_ids()),
            State::Day(ref day) => Some(day.secret_ids()),
            State::Complete(_) => None
        }
    }
}

impl<P: Eq + Hash> Default for State<P> {
    fn default() -> State<P> {
        State::Signups(Signups::default())
    }
}

/// A game which has not been started. The moderator may sign up players, or start the game.
pub struct Signups<P: Eq + Hash> {
    player_names: HashSet<P>
}

/// The possible errors returned by `Signups::start`.
#[derive(Debug)]
pub enum StartGameError {
    /// There are less than the required number of players.
    NotEnoughPlayers {
        /// This many players are required to start a game.
        required: usize,
        /// But only this many have signed up.
        found: usize
    },
    /// More roles than there are players have been specified.
    RolesCount {
        /// This many players have signed up.
        required: usize,
        /// But this many roles have been given.
        found: usize
    }
}

impl fmt::Display for StartGameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to start game: ")?;
        match *self {
            StartGameError::NotEnoughPlayers { required, found } => write!(f, "not enough players ({} required, {} signed up)", required, found),
            StartGameError::RolesCount { required, found } => write!(f, "too many roles ({} players, {} roles)", required, found),
        }
    }
}

impl<P: Eq + Hash> Signups<P> {
    /// Sign up a player. The `player_id` must be unique.
    ///
    /// Returns `true` if the player has been successfully signed up, or `false` if a player with that ID already exists.
    pub fn sign_up(&mut self, player_id: P) -> bool {
        self.player_names.insert(player_id)
    }

    /// Returns the number of players that have been signed up so far.
    pub fn num_players(&self) -> usize {
        self.player_names.len()
    }

    /// Start the game.
    ///
    /// If fewer roles than players are given, a number of Villagers equal to the difference will be added.
    pub fn start(self, roles: Vec<Role>) -> Result<State<P>, StartGameError> {
        let num_players = self.num_players();
        if num_players < MIN_PLAYERS {
            return Err(StartGameError::NotEnoughPlayers { required: MIN_PLAYERS, found: num_players });
        }
        if num_players < roles.len() {
            return Err(StartGameError::RolesCount { required: num_players, found: roles.len() });
        }
        let Signups { player_names } = self;
        let mut secret_ids = player_names.into_iter().collect::<Vec<_>>();
        thread_rng().shuffle(&mut secret_ids);
        let roles = roles.into_iter()
            .filter(|&role| role != Role::Villager)
            .fold((0, Vec::default()), |(mut num_ww, mut roles), role| {
                if let Role::Werewolf(_) = role {
                    roles.push(Role::Werewolf(num_ww));
                    num_ww += 1;
                } else {
                    roles.push(role);
                }
                (num_ww, roles)
            }).1;
        let multiverse = Multiverse::new(roles, num_players);
        // check for game-ending conditions
        if multiverse.game_over(false) {
            return Ok(State::Complete(Complete::new(secret_ids, multiverse)));
        }
        Ok(State::Night(Night {
            secret_ids, multiverse,
            last_heals: vec![None; num_players]
        }))
    }
}

impl<P: Eq + Hash> Default for Signups<P> {
    fn default() -> Signups<P> {
        Signups {
            player_names: HashSet::default()
        }
    }
}

impl<P: Eq + Hash> From<Signups<P>> for State<P> {
    fn from(state: Signups<P>) -> State<P> {
        State::Signups(state)
    }
}

/// A running game which is currently in night time, waiting for the players' night actions.
pub struct Night<P: Eq + Hash> {
    secret_ids: Vec<P>,
    last_heals: Vec<Option<usize>>,
    multiverse: Multiverse
}

impl<P: Eq + Hash> Night<P> {
    /*
    /// Advance the game state to the next day using natural action resolution.
    ///
    /// Takes night actions submitted by the players and processes them. Any mandatory night actions not submitted will be randomized.
    pub fn resolve_nar(mut self, night_actions: Vec<NightAction>) -> State<P> {
        unimplemented!(); //TODO
        // check for game-ending conditions
        if self.multiverse.game_over(false) {
            return State::Complete(Complete::new(self.secret_ids, self.multiverse));
        }
        State::Day(Day {
            secret_ids: self.secret_ids,
            multiverse: self.multiverse,
            night_action_results,
            last_heals: current_heals
        })
    }
    */

    /// Advance the game state to the next day using temporal action resolution.
    ///
    /// To do this, all night actions have to be submitted. The functions passed as arguments are used to ask for night actions.
    pub fn resolve_tar<H, I, W>(mut self, choose_heal_target: H, choose_investigation_target: I, choose_werewolf_kill_target: W) -> State<P> where
    H: Fn(&P, Vec<&P>) -> Option<P>,
    I: Fn(&P, Vec<&P>) -> Option<P>,
    W: Fn(&P, Vec<&P>) -> P {
        // reset kill lists
        for universe in self.multiverse.iter_mut() {
            universe.heals = Vec::default();
            universe.kills = Vec::default();
        }
        // healer actions
        let mut current_heals = vec![None; self.secret_ids.len()];
        if self.multiverse.role_alive(Role::Healer) {
            for (player_id, player) in shuffled_players(&self.secret_ids) {
                if !self.multiverse.alive().contains(&player_id) { continue; }
                let mut healable = {
                    let ids = &self.secret_ids;
                    self.multiverse.alive().into_iter()
                        .filter(|&iter_id| self.last_heals[player_id].map_or(true, |heal_id| heal_id != iter_id))
                        .map(|iter_id| &ids[iter_id])
                        .collect::<Vec<_>>()
                };
                thread_rng().shuffle(&mut healable);
                if let Some(target) = choose_heal_target(player, healable) {
                    let target_id = self.secret_ids.iter().position(|iter_player| &target == iter_player).expect("healed player not in game");
                    current_heals[player_id] = Some(target_id);
                    for universe in self.multiverse.iter_mut() {
                        let can_heal = universe.roles[player_id] == Role::Healer &&
                        universe.alive[player_id] &&
                        universe.alive[target_id];
                        if can_heal {
                            universe.heals.push(target_id);
                        }
                    }
                }
            }
        }
        // detective actions
        let mut night_action_results = vec![None; self.secret_ids.len()];
        if self.multiverse.role_alive(Role::Detective) {
            let all_players = shuffled_players(&self.secret_ids).into_iter()
                .map(|(_, player)| player)
                .collect::<Vec<_>>();
            for (player_id, player) in shuffled_players(&self.secret_ids) {
                if !self.multiverse.alive().contains(&player_id) { continue; }
                if let Some(target) = choose_investigation_target(player, all_players.clone()) {
                    let target_id = self.secret_ids.iter().position(|iter_player| &target == iter_player).expect("investigated player not in game");
                    let investigated_faction = if let Some(investigation_universe) = self.multiverse.iter()
                        .filter(|universe| universe.roles[player_id] == Role::Detective) // player must be detective,
                        .filter(|universe| universe.alive[player_id]) // and detective must be alive
                        .rand(&mut thread_rng())
                    {
                        investigation_universe.factions[target_id]
                    } else {
                        continue;
                    };
                    if night_action_results[player_id].is_some() { unimplemented!("multiple night action results for one player"); }
                    night_action_results[player_id] = Some(NightActionResult::Investigation(investigated_faction));
                    self.multiverse = self.multiverse.into_iter()
                        .filter(|universe| !(
                            universe.roles[player_id] == Role::Detective &&
                            universe.alive[player_id] &&
                            universe.factions[target_id] != investigated_faction
                        ))
                        .collect();
                }
            }
        }
        // werewolf kills
        {
            let mut alive = {
                let ids = &self.secret_ids;
                self.multiverse.alive().into_iter()
                    .map(|iter_id| &ids[iter_id])
                    .collect::<Vec<_>>()
            };
            thread_rng().shuffle(&mut alive);
            for (player_id, player) in shuffled_players(&self.secret_ids) {
                if !self.multiverse.alive().contains(&player_id) { continue; }
                let target = choose_werewolf_kill_target(player, alive.clone());
                let target_id = self.secret_ids.iter().position(|iter_player| &target == iter_player).expect("killed player not in game");
                for universe in self.multiverse.iter_mut() {
                    let can_kill = if let Role::Werewolf(werewolf_rank) = universe.roles[player_id] {
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
                    universe.alive[player_id] &&
                    universe.alive[target_id];
                    if can_kill {
                        universe.kill(target_id, true);
                    }
                }
            }
        }
        // kill all players on the death list
        for universe in self.multiverse.iter_mut() {
            for &player_id in &universe.kills {
                universe.alive[player_id] = false;
            }
        }
        self.multiverse.collapse_roles();
        // check for game-ending conditions
        if self.multiverse.game_over(false) {
            return State::Complete(Complete::new(self.secret_ids, self.multiverse));
        }
        State::Day(Day {
            secret_ids: self.secret_ids,
            multiverse: self.multiverse,
            night_action_results,
            last_heals: current_heals
        })
    }

    /// Returns the player list, sorted by secret player ID.
    pub fn secret_ids(&self) -> &[P] {
        &self.secret_ids
    }
}

impl<P: Eq + Hash> From<Night<P>> for State<P> {
    fn from(state: Night<P>) -> State<P> {
        State::Night(state)
    }
}

/// A running game which is currently in day time, waiting for the result of the lynch vote.
pub struct Day<P: Eq + Hash> {
    secret_ids: Vec<P>,
    pub(crate) multiverse: Multiverse,
    night_action_results: Vec<Option<NightActionResult>>,
    last_heals: Vec<Option<usize>>
}

impl<P: Eq + Hash> Day<P> {
    /// Contains results of the last night's night actions.
    pub fn night_action_results(&self) -> Vec<(&P, NightActionResult)> {
        let mut list = Vec::default();
        for (player_idx, result) in self.night_action_results.iter().enumerate() {
            if let &Some(result) = result {
                list.push((&self.secret_ids[player_idx], result));
            }
        }
        list
    }

    /// Tests whether `lynch` will panic.
    pub fn can_lynch(&self, lynch_target: &P) -> bool {
        match self.secret_ids.iter().position(|iter_player| lynch_target == iter_player) {
            Some(lynch_id) => self.multiverse.alive().contains(&lynch_id),
            None => false
        }
    }

    /// Advance the game state to the next night by lynching a player.
    ///
    /// See also `no_lynch`.
    pub fn lynch(mut self, lynch_target: P) -> State<P> {
        let lynch_id = self.secret_ids.iter().position(|iter_player| &lynch_target == iter_player).expect("lynched player not in game");
        // eliminate impossible gamestates (where the player to be killed by the vote is already dead), then kill voted player
        self.multiverse = self.multiverse.into_iter()
            .filter(|universe| universe.alive[lynch_id])
            .map(|mut universe| {
                universe.kill(lynch_id, false);
                universe
            })
            .collect();
        self.multiverse.collapse_roles();
        // check for game-ending conditions
        if self.multiverse.game_over(false) {
            return State::Complete(Complete::new(self.secret_ids, self.multiverse));
        }
        State::Night(Night {
            secret_ids: self.secret_ids,
            multiverse: self.multiverse,
            last_heals: self.last_heals
        })
    }

    /// Advance the game state to the next night without lynching any players.
    ///
    /// See also `lynch`.
    pub fn no_lynch(self) -> State<P> {
        // check for game-ending conditions
        if self.multiverse.game_over(false) {
            return State::Complete(Complete::new(self.secret_ids, self.multiverse));
        }
        State::Night(Night {
            secret_ids: self.secret_ids,
            multiverse: self.multiverse,
            last_heals: self.last_heals
        })
    }

    /// Returns the player list, sorted by secret player ID.
    pub fn secret_ids(&self) -> &[P] {
        &self.secret_ids
    }
}

impl<P: Eq + Hash> From<Day<P>> for State<P> {
    fn from(state: Day<P>) -> State<P> {
        State::Day(state)
    }
}

/// A completed game.
pub struct Complete<P: Eq + Hash> {
    /// The set of players who have won this game.
    pub winners: HashSet<P>
}

impl<P: Eq + Hash> Complete<P> {
    fn new(secret_ids: Vec<P>, multiverse: Multiverse) -> Complete<P> {
        if let Some(universe) = multiverse.into_iter().rand(&mut thread_rng()) {
            let winners = secret_ids.into_iter()
                .enumerate()
                .filter(|&(player_idx, _)| universe.factions[player_idx].wincon(&universe))
                .map(|(_, name)| name)
                .collect();
            Complete { winners }
        } else {
            Complete {
                winners: HashSet::default()
            }
        }
    }
}

impl<P: Eq + Hash> From<Complete<P>> for State<P> {
    fn from(state: Complete<P>) -> State<P> {
        State::Complete(state)
    }
}

/// Iterate over all players in a random order.
fn shuffled_players<P>(secret_ids: &[P]) -> Vec<(usize, &P)> {
    let mut result = secret_ids.iter().enumerate().collect::<Vec<_>>();
    thread_rng().shuffle(&mut result);
    result
}
