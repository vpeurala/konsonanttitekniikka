//! Where the save file lives and reading and writing it: the impure edge
//! of saving. What is in it is `Progress`, in the core crate.

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

pub use lukuloitsu_core::progress::Progress;

/// Where the save file lives, if saving is possible on this platform.
#[cfg(not(target_arch = "wasm32"))]
fn path() -> Option<PathBuf> {
    if cfg!(target_os = "android") {
        // The app's private storage, which Android gives every app.
        Some(PathBuf::from(
            "/data/data/fi.lukuloitsu.lukuloitsu/files/save.txt",
        ))
    } else if cfg!(any(target_os = "macos", target_os = "ios")) {
        // On iOS, HOME is the app's own sandbox.
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join("Library/Application Support/Lukuloitsu/save.txt"))
    } else if cfg!(target_family = "unix") || cfg!(target_os = "windows") {
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("APPDATA"))?;
        Some(PathBuf::from(home).join(".lukuloitsu/save.txt"))
    } else {
        None
    }
}

/// Loads the saved progress, or the defaults if there is none.
pub fn load() -> Progress {
    read_text()
        .map(|text| Progress::from_text(&text))
        .unwrap_or_default()
}

/// Saves progress. Failing to save is not worth stopping the game for, so
/// errors are only logged.
pub fn store(text: &str) {
    write_text(text);
}

#[cfg(not(target_arch = "wasm32"))]
fn read_text() -> Option<String> {
    std::fs::read_to_string(path()?).ok()
}

/// Writes a new file and then swaps it in, so an interrupted save never
/// leaves a half-written file behind.
#[cfg(not(target_arch = "wasm32"))]
fn write_text(text: &str) {
    let Some(path) = path() else {
        return;
    };
    let Some(dir) = path.parent() else {
        return;
    };
    let temporary = path.with_extension("tmp");
    let result = std::fs::create_dir_all(dir)
        .and_then(|_| std::fs::write(&temporary, text))
        .and_then(|_| std::fs::rename(&temporary, &path));
    if let Err(error) = result {
        macroquad::logging::warn!("Saving progress to {} failed: {error}", path.display());
    }
}

// In a browser, progress is kept in the page's local storage.
#[cfg(target_arch = "wasm32")]
fn read_text() -> Option<String> {
    crate::platform::web::load()
}

#[cfg(target_arch = "wasm32")]
fn write_text(text: &str) {
    crate::platform::web::store(text);
}
