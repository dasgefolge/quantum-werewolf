//! This is a Rust implementation of [Quantum Werewolf](https://web.archive.org/web/20181116123708/https://puzzle.cisra.com.au/2008/quantumwerewolf.html).

#![deny(missing_docs, rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

pub mod game;
pub mod handler;
pub mod player;
mod util;

pub use self::{
    handler::Handler,
    player::Player
};
