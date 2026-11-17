//! `unsafe` is denied in every crate; only the web storage bridge may allow it, because calling
//! JavaScript from wasm needs it. This finds any other crate that turns the lint off.

const ALLOWED: &str = "crates/flappy-web-storage/";

/// The files, among `(path, content)` pairs, that mention the `unsafe_code` lint outside the
/// one crate allowed to.
pub fn violations(files: &[(String, String)]) -> Vec<String> {
    files
        .iter()
        .filter(|(path, content)| !path.starts_with(ALLOWED) && content.contains("unsafe_code"))
        .map(|(path, _)| path.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, content: &str) -> (String, String) {
        (path.to_owned(), content.to_owned())
    }

    #[test]
    fn crates_that_never_mention_unsafe_code_are_fine() {
        let files = [file("crates/flappy-core/src/lib.rs", "pub fn f() {}")];

        assert_eq!(violations(&files), Vec::<String>::new());
    }

    #[test]
    fn allowing_unsafe_code_outside_the_web_storage_bridge_is_reported() {
        let files = [
            file("crates/flappy-core/src/lib.rs", "#![allow(unsafe_code)]"),
            file("crates/flappy-client/src/main.rs", "#[expect(unsafe_code)]"),
        ];

        assert_eq!(
            violations(&files),
            vec![
                "crates/flappy-core/src/lib.rs".to_owned(),
                "crates/flappy-client/src/main.rs".to_owned(),
            ]
        );
    }

    #[test]
    fn overriding_the_lint_in_a_crate_manifest_is_reported() {
        let files = [file(
            "crates/flappy-storage/Cargo.toml",
            "[lints.rust]\nunsafe_code = \"allow\"",
        )];

        assert_eq!(
            violations(&files),
            vec!["crates/flappy-storage/Cargo.toml".to_owned()]
        );
    }

    #[test]
    fn the_web_storage_bridge_may_allow_unsafe_code() {
        let files = [file(
            "crates/flappy-web-storage/src/lib.rs",
            "#![allow(unsafe_code)]",
        )];

        assert_eq!(violations(&files), Vec::<String>::new());
    }
}
