# Vendored crates

## miniquad 0.4.11

A copy of [miniquad](https://github.com/not-fl3/miniquad) 0.4.11, macroquad's
platform layer, used through `[patch.crates-io]` in `Cargo.toml`. Changes
from the published crate:

- `src/native/android/mod_inject.rs`: `#[no_mangle]` became
  `#[unsafe(no_mangle)]`. `cargo quad-apk` pastes this file into the game's
  own source, and Rust 2024 requires the `unsafe(...)` form there.
- `java/MainActivity.java`: the volume buttons are passed on to Android
  instead of being swallowed, and they adjust media volume while the game
  is open.
