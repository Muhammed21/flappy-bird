use std::path::{Path, PathBuf};

use flappy_storage::{FileStore, MAX_SAVE_BYTES, Store, data_dir};

/// A fresh, empty folder for one test, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("flappy-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |name| {
        pairs
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| (*value).to_owned())
    }
}

#[test]
fn with_nothing_saved_there_is_nothing_to_read() {
    let scratch = Scratch::new("empty");

    assert_eq!(FileStore::new(scratch.path()).read(), None);
}

#[test]
fn a_written_save_reads_back_even_in_a_folder_that_did_not_exist() {
    let scratch = Scratch::new("roundtrip");
    let store = FileStore::new(scratch.path().join("nested"));

    store.write("hello").unwrap();

    assert_eq!(store.read(), Some("hello".to_owned()));
}

#[test]
fn a_new_save_replaces_the_old_one_and_leaves_no_temporary_file() {
    let scratch = Scratch::new("replace");
    let store = FileStore::new(scratch.path());

    store.write("a much longer first save").unwrap();
    store.write("second").unwrap();

    assert_eq!(store.read(), Some("second".to_owned()));
    let files: Vec<_> = std::fs::read_dir(scratch.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(files, vec![std::ffi::OsString::from("best.json")]);
}

#[test]
fn an_oversized_file_is_not_read_into_memory() {
    let scratch = Scratch::new("oversized");
    let store = FileStore::new(scratch.path());
    store.write("x").unwrap();
    std::fs::write(
        scratch.path().join("best.json"),
        vec![b' '; MAX_SAVE_BYTES + 1],
    )
    .unwrap();

    assert_eq!(store.read(), None);
}

#[test]
fn flappy_home_overrides_the_data_folder_so_dev_runs_stay_away_from_real_saves() {
    let found = data_dir(
        "macos",
        env(&[("FLAPPY_HOME", "/tmp/dev"), ("HOME", "/Users/me")]),
    );

    assert_eq!(found, Some(PathBuf::from("/tmp/dev")));
}

#[test]
fn macos_saves_in_application_support() {
    let found = data_dir("macos", env(&[("HOME", "/Users/me")]));

    assert_eq!(
        found,
        Some(PathBuf::from(
            "/Users/me/Library/Application Support/flappy-bird"
        ))
    );
}

#[test]
fn linux_saves_in_xdg_data_home_or_else_in_local_share() {
    assert_eq!(
        data_dir(
            "linux",
            env(&[("XDG_DATA_HOME", "/data"), ("HOME", "/home/me")])
        ),
        Some(PathBuf::from("/data/flappy-bird"))
    );
    assert_eq!(
        data_dir("linux", env(&[("HOME", "/home/me")])),
        Some(PathBuf::from("/home/me/.local/share/flappy-bird"))
    );
}

#[test]
fn windows_saves_in_appdata() {
    let found = data_dir(
        "windows",
        env(&[("APPDATA", r"C:\Users\me\AppData\Roaming")]),
    );

    assert_eq!(
        found,
        Some(PathBuf::from(r"C:\Users\me\AppData\Roaming").join("flappy-bird"))
    );
}

#[test]
fn empty_or_relative_locations_are_ignored() {
    assert_eq!(
        data_dir(
            "linux",
            env(&[("XDG_DATA_HOME", "relative"), ("HOME", "/home/me")])
        ),
        Some(PathBuf::from("/home/me/.local/share/flappy-bird"))
    );
    assert_eq!(
        data_dir("linux", env(&[("FLAPPY_HOME", ""), ("HOME", "")])),
        None
    );
    assert_eq!(data_dir("macos", env(&[])), None);
}
