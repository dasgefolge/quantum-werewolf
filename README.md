This is a [Rust](https://www.rust-lang.org/) implementation of [**Quantum Werewolf**](http://puzzle.cisra.com.au/2008/quantumwerewolf.html), a variant of [Werewolf](https://en.wikipedia.org/wiki/Mafia_%28party_game%29) based on [quantum mechanics](https://en.wikipedia.org/wiki/Quantum_mechanics). It can be used as a standalone program, or as a Rust library.

# Usage

Currently, the only way to run a game of Quantum Werewolf using only this code requires one person to not participate in the game and instead act as a moderator. A moderator-less mode is planned.

1. Install [Rust](https://www.rust-lang.org/).
2. Run `cargo install --git=https://github.com/dasgefolge/quantum-werewolf`.
3. Run `qww`. This will display different kinds of messages:
    * Messages starting with `[ ** ]` are public messages. You should read or show them to all players.
    * Messages starting with `[ __ ]` are private messages. You should make sure only the indicated player sees them.
    * Messages starting with `[ ?? ]` are questions. If a question is for one player, you should ask them secretly. The town lynch target, on the other hand, is determined by all living players, like in a regular game of Werewolf.
    * Messages starting with `[ !! ]` are errors. If you see one, try again.
