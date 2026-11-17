use std::io::{Read, Write};
use std::path::PathBuf;

use crate::{MAX_SAVE_BYTES, Store};

const FILE: &str = "best.json";
const TEMPORARY: &str = "best.json.tmp";
const APP: &str = "flappy-bird";

/// The save as a file. Writes go to a temporary file first and are renamed over the save, so a
/// crash or a full disk mid-write leaves the previous save intact.
#[derive(Clone, Debug)]
pub struct FileStore {
    dir: PathBuf,
}

impl FileStore {
    #[must_use]
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }
}

impl Store for FileStore {
    fn read(&self) -> Option<String> {
        let file = std::fs::File::open(self.dir.join(FILE)).ok()?;
        let limit = u64::try_from(MAX_SAVE_BYTES).ok()?.saturating_add(1);
        let mut text = String::new();
        file.take(limit).read_to_string(&mut text).ok()?;
        (text.len() <= MAX_SAVE_BYTES).then_some(text)
    }

    fn write(&self, text: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let temporary = self.dir.join(TEMPORARY);
        let mut file = std::fs::File::create(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&temporary, self.dir.join(FILE))
    }
}

/// Where the game keeps its save on `os` (`std::env::consts::OS`), reading the environment
/// through `env`. `FLAPPY_HOME` overrides it, so tests and dev runs never touch a real save.
#[must_use]
pub fn data_dir(os: &str, env: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    let set = |name: &str| {
        env(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    };
    if let Some(home) = set("FLAPPY_HOME") {
        return Some(home);
    }
    let base = match os {
        "macos" => set("HOME").map(|home| home.join("Library").join("Application Support")),
        "windows" => set("APPDATA"),
        _ => set("XDG_DATA_HOME")
            .filter(|dir| dir.is_absolute())
            .or_else(|| set("HOME").map(|home| home.join(".local").join("share"))),
    };
    base.map(|dir| dir.join(APP))
}
