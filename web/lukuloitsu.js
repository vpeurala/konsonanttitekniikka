// What Lukuloitsu needs from the page, called from src/web.rs: saving
// progress in the browser's local storage, and telling whether the screen
// is a touch screen, taking away the loading message once it is ready, and
// counting events in the visitor statistics.
(function () {
    const KEY = "lukuloitsu-save";
    // The save text as bytes, between reporting its length and copying it.
    let pending = new Uint8Array(0);

    miniquad_add_plugin({
        name: "lukuloitsu",
        // Must match PAGE_VERSION in src/web.rs.
        version: 2,
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
            // Counts an event in the visitor statistics, unless GoatCounter
            // hasn't loaded or is blocked.
            importObject.env.lukuloitsu_event = function (path, pathLen, title, titleLen) {
                const text = function (ptr, len) {
                    return new TextDecoder().decode(new Uint8Array(wasm_memory.buffer, ptr, len));
                };
                const counter = window.goatcounter;
                if (counter && counter.count) {
                    counter.count({ path: text(path, pathLen), title: text(title, titleLen), event: true });
                }
            };
            // A phone or tablet: its main pointer is a finger.
            importObject.env.lukuloitsu_touch_screen = function () {
                return window.matchMedia("(pointer: coarse)").matches ? 1 : 0;
            };
        },
    });
})();
