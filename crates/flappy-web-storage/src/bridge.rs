use flappy_storage::{MAX_SAVE_BYTES, Store};

/// Must match `version` in web/storage.js; miniquad reports a mismatch in the console.
const BRIDGE_VERSION: u32 = 1;
const KEY: &str = "flappy-bird.best";

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    /// Looks `key` up and keeps the value aside; returns its length in bytes, or -1 if absent.
    fn flappy_storage_get(key: *const u8, key_len: usize) -> i32;
    /// Copies the value kept aside by the last get into `len` bytes at `dest`, then forgets it.
    fn flappy_storage_take(dest: *mut u8, len: usize);
    /// Stores `value` under `key`; returns 1 when stored, 0 when the browser refused.
    fn flappy_storage_set(
        key: *const u8,
        key_len: usize,
        value: *const u8,
        value_len: usize,
    ) -> i32;
}

/// Read by miniquad's plugin loader to check that the JS half is the same version.
#[unsafe(no_mangle)]
pub extern "C" fn flappy_storage_crate_version() -> u32 {
    BRIDGE_VERSION
}

/// The best game saved in the browser's local storage.
#[derive(Clone, Copy, Debug, Default)]
pub struct WebStore;

impl Store for WebStore {
    fn read(&self) -> Option<String> {
        // SAFETY: the pointer and length describe `KEY`, a live &'static str; JS only reads it.
        let len = unsafe { flappy_storage_get(KEY.as_ptr(), KEY.len()) };
        let len = usize::try_from(len)
            .ok()
            .filter(|len| *len <= MAX_SAVE_BYTES)?;
        let mut buffer = vec![0_u8; len];
        // SAFETY: `buffer` owns exactly `len` writable bytes for the whole call, and JS writes at
        // most `len` bytes into it.
        unsafe { flappy_storage_take(buffer.as_mut_ptr(), buffer.len()) };
        String::from_utf8(buffer).ok()
    }

    fn write(&self, text: &str) -> std::io::Result<()> {
        // SAFETY: both pointer and length pairs describe live string slices for the whole call;
        // JS only reads them.
        let stored =
            unsafe { flappy_storage_set(KEY.as_ptr(), KEY.len(), text.as_ptr(), text.len()) };
        if stored == 1 {
            Ok(())
        } else {
            Err(std::io::Error::other(
                "the browser refused to store the save",
            ))
        }
    }
}
