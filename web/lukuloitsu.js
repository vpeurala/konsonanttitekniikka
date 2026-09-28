// What Lukuloitsu needs from the page, called from src/web.rs: saving
// progress in the browser's local storage, and telling whether the screen
// is a touch screen, and taking away the loading message once it is ready.
(function () {
    const KEY = "lukuloitsu-save";
    // The save text as bytes, between reporting its length and copying it.
    let pending = new Uint8Array(0);

    miniquad_add_plugin({
        name: "lukuloitsu",
        // Must match PAGE_VERSION in src/web.rs.
        version: 1,
        register_plugin: function (importObject) {
            importObject.env.lukuloitsu_save_len = function () {
                let text = null;
                try {
                    text = window.localStorage.getItem(KEY);
                } catch (e) {
                    // Storage is blocked, as in some private windows.
                }
                pending = new TextEncoder().encode(text || "");
                return pending.length;
            };
            importObject.env.lukuloitsu_save_read = function (ptr) {
                new Uint8Array(wasm_memory.buffer, ptr, pending.length).set(pending);
                pending = new Uint8Array(0);
            };
            importObject.env.lukuloitsu_save_write = function (ptr, len) {
                const bytes = new Uint8Array(wasm_memory.buffer, ptr, len);
                try {
                    window.localStorage.setItem(KEY, new TextDecoder().decode(bytes));
                } catch (e) {
                    console.warn("Saving progress failed", e);
                }
            };
            importObject.env.lukuloitsu_loaded = function () {
                const loading = document.getElementById("loading");
                if (loading) {
                    loading.remove();
                }
            };
            // A phone or tablet: its main pointer is a finger.
            importObject.env.lukuloitsu_touch_screen = function () {
                return window.matchMedia("(pointer: coarse)").matches ? 1 : 0;
            };
        },
    });
})();
