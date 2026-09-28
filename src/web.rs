//! What the game needs from the web page when it runs in a browser:
//! saving in the browser's local storage, and whether the screen is a
//! touch screen, telling it when loading is done, and counting events for
//! its visitor statistics. The functions are in
//! `web/lukuloitsu.js`.

/// The version of the functions below. `web/lukuloitsu.js` states the same
/// number, and the page complains if they differ, as when a browser has
/// kept an old copy of one of the files.
const PAGE_VERSION: u32 = 2;

#[unsafe(no_mangle)]
pub extern "C" fn lukuloitsu_crate_version() -> u32 {
    PAGE_VERSION
}

mod page {
    unsafe extern "C" {
        pub fn lukuloitsu_save_len() -> u32;
        pub fn lukuloitsu_save_read(ptr: *mut u8);
        pub fn lukuloitsu_save_write(ptr: *const u8, len: u32);
        pub fn lukuloitsu_touch_screen() -> u32;
        pub fn lukuloitsu_loaded();
        pub fn lukuloitsu_event(path: *const u8, path_len: u32, title: *const u8, title_len: u32);
    }
}

/// The saved text, if any.
pub fn load() -> Option<String> {
    // SAFETY: the page copies exactly the length it reported into the
    // buffer.
    let bytes = unsafe {
        let len = page::lukuloitsu_save_len() as usize;
        if len == 0 {
            return None;
        }
        let mut bytes = vec![0u8; len];
        page::lukuloitsu_save_read(bytes.as_mut_ptr());
        bytes
    };
    String::from_utf8(bytes).ok()
}

pub fn store(text: &str) {
    // SAFETY: the page only reads the given bytes, during the call.
    unsafe { page::lukuloitsu_save_write(text.as_ptr(), text.len() as u32) }
}

/// Tells the page the game is ready, so it can take away its loading
/// message.
pub fn loaded() {
    // SAFETY: takes nothing and returns nothing.
    unsafe { page::lukuloitsu_loaded() }
}

/// Hands an event to the page's visitor statistics.
pub fn event(path: &str, title: &str) {
    // SAFETY: the page only reads the given bytes, during the call.
    unsafe {
        page::lukuloitsu_event(
            path.as_ptr(),
            path.len() as u32,
            title.as_ptr(),
            title.len() as u32,
        )
    }
}

/// Whether the page is shown on a touch screen without a mouse, as on a
/// phone or tablet.
pub fn touch_screen() -> bool {
    // SAFETY: takes nothing and returns a number.
    unsafe { page::lukuloitsu_touch_screen() != 0 }
}
