//! This is a Rust implementation of [Quantum Werewolf](http://puzzle.cisra.com.au/2008/quantumwerewolf.html).

#![cfg_attr(test, deny(warnings))]
#![warn(trivial_casts)]
#![deny(missing_docs, unused)]
#![forbid(unused_extern_crates, unused_import_braces)]

extern crate rand;

pub mod game;
pub mod player;
mod util;

pub use player::Player;