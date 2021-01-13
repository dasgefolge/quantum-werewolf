//! This is a Rust implementation of [Quantum Werewolf](https://web.archive.org/web/20181116123708/https://puzzle.cisra.com.au/2008/quantumwerewolf.html).

#![cfg_attr(test, deny(warnings))]
#![warn(trivial_casts)]
#![deny(missing_docs, unused, unused_qualifications)]
#![forbid(unused_import_braces)]

pub mod game;
pub mod handler;
pub mod player;
mod util;

pub use self::{
    handler::Handler,
    player::Player
};
