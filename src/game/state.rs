//! Game state representation.

use std::{
    collections::HashSet,
    fmt,
    hash::Hash
};
use rand::prelude::*;
use serde_derive::{
    Deserialize,
    Serialize
};
use crate::{
    game::{
        Faction,
        Multiverse,
        NightAction,
        NightActionResult,
        Role
    },
    util::QwwIteratorExt
};

/// The minimum number of players required to start a game.
pub const MIN_PLAYERS: usize = 3;

/// This enum represents the state of the game. Each variant contains relevant methods to observe or progress the game state, refer to their documentation for details.
///
/// The type parameter `P` is used for player identifiers.
#[derive(Debug, Serialize, Deserialize)]
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
            State::Night(ref night) => Some(night.alive()),
            State::Day(ref day) => Some(day.alive()),
            State::Complete(_) => None
        }
    }

    /// Returns the number of players that are in this game.
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
#[derive(Debug, Serialize, Deserialize)]
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

    /// Returns `true` if the given player is already signed up.
    pub fn is_signed_up(&self, player_id: &P) -> bool {
        self.player_names.contains(player_id)
    }

    /// Returns the number of players that have been signed up so far.
    pub fn num_players(&self) -> usize {
        self.player_names.len()
    }

    /// Removes a player from the signups.
    ///
    /// Returns `true` if the player was previously signed up.
    pub fn remove_player(&mut self, player_id: &P) -> bool {
        self.player_names.remove(player_id)
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
        secret_ids.shuffle(&mut thread_rng());
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Night<P: Eq + Hash> {
    secret_ids: Vec<P>,
    last_heals: Vec<Option<usize>>,
    multiverse: Multiverse
}

impl<P: Eq + Hash> Night<P> {
    /// Returns `true` if no more night actions can be submitted.
    pub fn actions_complete(&self, night_actions: &[NightAction<P>]) -> bool {
        true && // required to make the if blocks parse as expressions for some reason
        if self.multiverse.role_alive(Role::Healer) {
            // all healer actions
            self.multiverse.alive().into_iter().all(|player_idx|
                night_actions.into_iter().any(|action| if let NightAction::Heal(ref src, _) = *action {
                    &self.secret_ids[player_idx] == src
                } else {
                    false
                })
            )
        } else { true } &&
        if self.multiverse.role_alive(Role::Detective) {
            // all detective investigations
            self.multiverse.alive().into_iter().all(|player_idx|
                night_actions.into_iter().any(|action| if let NightAction::Investigate(ref src, _) = *action {
                    &self.secret_ids[player_idx] == src
                } else {
                    false
                })
            )
        } else { true } &&
        // all werewolf kills
        self.multiverse.alive().into_iter().all(|player_idx|
            night_actions.into_iter().any(|action| if let NightAction::Kill(ref src, _) = *action {
                &self.secret_ids[player_idx] == src
            } else {
                false
            })
        )
    }

    /// Returns the set of players which are still alive in at least one possible universe.
    pub fn alive(&self) -> HashSet<&P> {
        self.multiverse.alive().into_iter()
            .map(|idx| &self.secret_ids[idx])
            .collect()
    }

    /// Advance the game state to the next day using natural action resolution.
    ///
    /// Takes night actions submitted by the players and processes them. Any mandatory night actions not submitted will be randomized.
    pub fn resolve_nar(mut self, night_actions: &[NightAction<P>]) -> State<P> {
        // reset kill lists
        for universe in self.multiverse.iter_mut() {
            universe.heals = Vec::default();
            universe.kills = Vec::default();
        }
        // resolve night actions
        let mut current_heals = vec![None; self.secret_ids.len()];
        let mut night_action_results = vec![None; self.secret_ids.len()];
        for action in self.sanitized_night_actions(night_actions) { // healer/detective/werewolf setup does not have any dependencies, so resolve in submitted order
            match action {
                NightAction::Heal(src_idx, tgt_idx) => {
                    current_heals[src_idx] = Some(tgt_idx);
                    for universe in self.multiverse.iter_mut() {
                        let can_heal = universe.roles[src_idx] == Role::Healer &&
                        universe.alive[src_idx] &&
                        universe.alive[tgt_idx];
                        if can_heal {
                            universe.heal(tgt_idx);
                        }
                    }
                }
                NightAction::Investigate(src_idx, tgt_idx) => {
                    let investigated_faction = if let Some(investigation_universe) = self.multiverse.iter()
                        .filter(|universe| universe.roles[src_idx] == Role::Detective) // player must be detective,
                        .filter(|universe| universe.alive[src_idx]) // and detective must be alive
                        .rand(&mut thread_rng())
                    {
                        investigation_universe.factions[tgt_idx]
                    } else {
                        continue;
                    };
                    if night_action_results[src_idx].is_some() { unimplemented!("multiple night action results for one player"); }
                    night_action_results[src_idx] = Some(NightActionResult::Investigation(investigated_faction));
                    self.multiverse = self.multiverse.into_iter()
                        .filter(|universe| !(
                            universe.roles[src_idx] == Role::Detective &&
                            universe.alive[src_idx] &&
                            universe.factions[tgt_idx] != investigated_faction
                        ))
                        .collect();
                }
                NightAction::Kill(src_idx, tgt_idx) => {
                    for universe in self.multiverse.iter_mut() {
                        let can_kill = if let Role::Werewolf(werewolf_rank) = universe.roles[src_idx] {
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
                        universe.alive[src_idx] &&
                        universe.alive[tgt_idx];
                        if can_kill {
                            universe.kill(tgt_idx, true);
                        }
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
                healable.shuffle(&mut thread_rng());
                if let Some(target) = choose_heal_target(player, healable) {
                    let target_id = self.secret_ids.iter().position(|iter_player| &target == iter_player).expect("healed player not in game");
                    current_heals[player_id] = Some(target_id);
                    for universe in self.multiverse.iter_mut() {
                        let can_heal = universe.roles[player_id] == Role::Healer &&
                        universe.alive[player_id] &&
                        universe.alive[target_id];
                        if can_heal {
                            universe.heal(target_id);
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
            alive.shuffle(&mut thread_rng());
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

    /// Remove illegal actions, add missing compulsory actions.
    fn sanitized_night_actions(&self, night_actions: &[NightAction<P>]) -> Vec<NightAction<usize>> {
        let mut result = Vec::default();
        // remove illegal actions
        for action in night_actions {
            match *action {
                NightAction::Heal(ref src, ref tgt) => {
                    let src_idx = if let Some(idx) = self.secret_ids.iter().position(|iter_player| src == iter_player) { idx } else { continue; };
                    let tgt_idx = if let Some(idx) = self.secret_ids.iter().position(|iter_player| tgt == iter_player) { idx } else { continue; };
                    let alive = self.alive();
                    if !alive.contains(&src) || !alive.contains(&tgt) { continue; }
                    if result.iter().any(|action| if let NightAction::Heal(ref iter_src, _) = *action { *iter_src == src_idx } else { false }) { continue; }
                    if self.last_heals[src_idx] != Some(tgt_idx) {
                        result.push(NightAction::Heal(src_idx, tgt_idx));
                    }
                }
                NightAction::Investigate(ref src, ref tgt) => {
                    let src_idx = if let Some(idx) = self.secret_ids.iter().position(|iter_player| src == iter_player) { idx } else { continue; };
                    let tgt_idx = if let Some(idx) = self.secret_ids.iter().position(|iter_player| tgt == iter_player) { idx } else { continue; };
                    if !self.alive().contains(&src) { continue; }
                    if result.iter().any(|action| if let NightAction::Investigate(ref iter_src, _) = *action { *iter_src == src_idx } else { false }) { continue; }
                    result.push(NightAction::Investigate(src_idx, tgt_idx));
                }
                NightAction::Kill(ref src, ref tgt) => {
                    let src_idx = if let Some(idx) = self.secret_ids.iter().position(|iter_player| src == iter_player) { idx } else { continue; };
                    let tgt_idx = if let Some(idx) = self.secret_ids.iter().position(|iter_player| tgt == iter_player) { idx } else { continue; };
                    let alive = self.alive();
                    if !alive.contains(&src) || !alive.contains(&tgt) { continue; }
                    if result.iter().any(|action| if let NightAction::Kill(ref iter_src, _) = *action { *iter_src == src_idx } else { false }) { continue; }
                    result.push(NightAction::Kill(src_idx, tgt_idx));
                }
            }
        }
        // add missing compulsory actions
        for secret_id in 0..self.secret_ids.len() {
            // werewolf kill
            if self.multiverse.alive().contains(&secret_id) && !result.iter().any(|action| if let &NightAction::Kill(src_idx, _) = action { src_idx == secret_id } else { false }) {
                if let Some(random_id) = self.multiverse.alive().into_iter().rand(&mut thread_rng()) {
                    result.push(NightAction::Kill(secret_id, random_id));
                }
            }
        }
        result
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Day<P: Eq + Hash> {
    secret_ids: Vec<P>,
    multiverse: Multiverse,
    night_action_results: Vec<Option<NightActionResult>>,
    last_heals: Vec<Option<usize>>
}

impl<P: Eq + Hash> Day<P> {
    /// Returns the set of players which are still alive in at least one possible universe.
    pub fn alive(&self) -> HashSet<&P> {
        self.multiverse.alive().into_iter()
            .map(|idx| &self.secret_ids[idx])
            .collect()
    }

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

    /// Produces the anonymized probability table shown to players at the start of the day.
    ///
    /// For each player in `secret_id` order, returns that player's probability of being town, of being a werewolf, and of being dead, if that player can still be alive. Otherwise, returns that player's faction.
    pub fn probability_table(&self) -> Vec<Result<(f64, f64, f64), Faction>> {
        self.multiverse.probability_table()
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
#[derive(Debug, Serialize, Deserialize)]
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
    result.shuffle(&mut thread_rng());
    result
}
