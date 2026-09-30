//! Where a frame's input comes from: the keyboard, touches and the app
//! being in the background. `frame` makes the `Frame` the rest of the app
//! sees (defined in the core crate).

pub mod frame;
pub mod keyboard;
pub mod lifecycle;
pub mod touch;
