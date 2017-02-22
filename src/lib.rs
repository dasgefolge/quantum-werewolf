//! This is a Rust implementation of [Quantum Werewolf](http://puzzle.cisra.com.au/2008/quantumwerewolf.html).

#![cfg_attr(test, deny(warnings))]
#![warn(trivial_casts)]
#![deny(missing_docs)]
#![forbid(unused, unused_extern_crates, unused_import_braces)]

#[macro_use] extern crate itertools;

mod game;
pub mod player;

pub use game::Game;
