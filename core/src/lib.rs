//! The rules and state machines of Lukuloitsu: the pairs, memory, badges,
//! the save file's format and the game itself.
//!
//! Nothing here draws, plays a sound or asks for the time or for input.
//! Whatever the outside world has to say comes in as arguments, and
//! whatever this wants done goes out as return values. The crate has no
//! graphics or audio dependency, so the compiler keeps it that way.

#![cfg_attr(test, allow(clippy::float_cmp))]

pub mod arena;
pub mod badges;
pub mod color;
pub mod curriculum;
pub mod effects;
pub mod game;
pub mod key;
pub mod levels;
pub mod long_numbers;
pub mod memory;
pub mod obstacles;
pub mod pairs;
pub mod portals;
pub mod progress;
pub mod rng;
pub mod save;
pub mod sfx;
