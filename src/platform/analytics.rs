//! Sending anonymous counts of how the game is played to the web version's
//! statistics at GoatCounter. What is counted, and the paths and titles it
//! is filed under, is `lukuloitsu_core::analytics::Event`. The apps send
//! nothing.

pub use lukuloitsu_core::analytics::Event;

/// Sends the count to the statistics.
pub fn send(event: &Event) {
    #[cfg(target_arch = "wasm32")]
    crate::platform::web::event(&event.path(), &event.title());
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
}
