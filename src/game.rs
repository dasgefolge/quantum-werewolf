//! Core game logic.

use std::collections::HashSet;

use rand::{Rng, thread_rng};

use player::Player;

/// The possible errors returned by `Game::new`.
#[derive(Debug)]
pub enum NewGameError {
    /// There are less than three players.
    NotEnoughPlayers,
    /// Multiple players have the same name.
    NameCollision(String)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Role {
    Detective,
    Villager,
    Werewolf(usize)
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Universe {
    roles: Vec<Role>
}

impl From<Vec<Role>> for Universe {
    fn from(roles: Vec<Role>) -> Universe {
        Universe {
            roles: roles
        }
    }
}

/// This represents the state of a game.
#[derive(Debug)]
pub struct Game {
    players: Vec<Box<Player>>,
    multiverse: HashSet<Universe>
}

impl Game {
    /// Creates a new game from a list of players.
    ///
    /// # Errors
    ///
    /// Will return an error if no game can be created with the given player list. See `NewGameError` for details.
    pub fn new(players: Vec<Box<Player>>) -> Result<Game, NewGameError> {
        if players.len() < 3 {
            return Err(NewGameError::NotEnoughPlayers);
        }
        for ((i1, p1), (i2, p2)) in iproduct!(players.iter().enumerate(), players.iter().enumerate()) {
            if i1 != i2 && p1.name() == p2.name() {
                return Err(NewGameError::NameCollision(p1.name().to_owned()));
            }
        }
        let num_ww = players.len() / 3;
        let roles: Vec<Role> = {
            let mut result: Vec<Role> = (0..num_ww).map(|i| Role::Werewolf(i)).collect();
            result.push(Role::Detective);
            result
        };
        let mut permutations = vec![vec![Role::Villager; players.len()]];
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
            players: players,
            multiverse: multiverse
        })
    }

    /// Runs the entire game and returns the names of the winners.
    pub fn run(mut self) -> HashSet<String> {
        thread_rng().shuffle(&mut self.players);
        for (i, player) in self.players.iter_mut().enumerate() {
            player.recv_id(i);
        }
        unimplemented!() //TODO game loop
    }
}
