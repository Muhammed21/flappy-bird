"use strict";
// The JS half of crates/flappy-web-storage: the save in localStorage. Loaded after gl.js.
(function () {
    const MAX_BYTES = 1024 * 1024;
    let pending = null;

    const view = (ptr, len) => new Uint8Array(wasm_memory.buffer, ptr, len);
    const read = (ptr, len) => new TextDecoder().decode(view(ptr, len));

    miniquad_add_plugin({
        name: "flappy_storage",
        version: 1,
        register_plugin: function (importObject) {
            importObject.env.flappy_storage_get = function (keyPtr, keyLen) {
                pending = null;
                let value = null;
                try {
                    value = window.localStorage.getItem(read(keyPtr, keyLen));
                } catch (error) {
                    return -1;
                }
                if (value === null) {
                    return -1;
                }
                const bytes = new TextEncoder().encode(value);
                if (bytes.length > MAX_BYTES) {
                    return -1;
                }
                pending = bytes;
                return bytes.length;
            };
            importObject.env.flappy_storage_take = function (destPtr, len) {
                if (pending !== null && pending.length === len) {
                    view(destPtr, len).set(pending);
                }
                pending = null;
            };
            importObject.env.flappy_storage_set = function (keyPtr, keyLen, valuePtr, valueLen) {
                try {
                    window.localStorage.setItem(read(keyPtr, keyLen), read(valuePtr, valueLen));
                    return 1;
                } catch (error) {
                    return 0;
                }
            };
        },
    });
})();
