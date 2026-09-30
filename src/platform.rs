//! Where the app meets the device: the save file, the web page's
//! statistics and the JavaScript bridge.

pub mod analytics;
pub mod save;
#[cfg(target_arch = "wasm32")]
pub mod web;
