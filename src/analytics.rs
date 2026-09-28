//! Anonymous counts of how the game is played, for the web version's
//! statistics at GoatCounter: games started, levels completed and so on.
//! Nothing identifies the player. The apps send nothing.

/// Counts one `path`, like "taso-lapaisty/5", described by `title`.
pub fn event(path: &str, title: &str) {
    #[cfg(target_arch = "wasm32")]
    crate::web::event(path, title);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (path, title);
}
