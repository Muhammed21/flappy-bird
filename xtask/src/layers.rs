//! The layer rule (docs/layers.md): a crate may depend only on crates of a lower layer.
//! Dev-dependencies are free, so tests may use anything.

use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Layer {
    Core,
    Replay,
    Storage,
    Platform,
    App,
    Tool,
}

const LAYERS: &[(&str, Layer)] = &[
    ("flappy-core", Layer::Core),
    ("flappy-replay", Layer::Replay),
    ("flappy-storage", Layer::Storage),
    ("flappy-web-storage", Layer::Platform),
    ("flappy-client", Layer::App),
    ("flappy-bench", Layer::Tool),
    ("xtask", Layer::Tool),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepKind {
    Normal,
    Build,
    Dev,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub kind: DepKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub deps: Vec<Dependency>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Violation {
    Unclassified { krate: String },
    Forbidden { from: String, to: String },
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unclassified { krate } => write!(
                f,
                "{krate} has no layer: add it to LAYERS in xtask/src/layers.rs and docs/layers.md"
            ),
            Self::Forbidden { from, to } => {
                write!(f, "{from} may not depend on {to} (docs/layers.md)")
            }
        }
    }
}

fn layer_of(name: &str) -> Option<Layer> {
    LAYERS
        .iter()
        .find(|(krate, _)| *krate == name)
        .map(|(_, layer)| *layer)
}

fn package_violations(package: &Package) -> impl Iterator<Item = Violation> + '_ {
    let from = layer_of(&package.name);
    package
        .deps
        .iter()
        .filter(|dep| dep.kind != DepKind::Dev)
        .filter(move |dep| match (from, layer_of(&dep.name)) {
            (Some(from), Some(to)) => to >= from,
            _ => false,
        })
        .map(|dep| Violation::Forbidden {
            from: package.name.clone(),
            to: dep.name.clone(),
        })
}

#[must_use]
pub fn check(members: &[Package]) -> Vec<Violation> {
    let unclassified = members
        .iter()
        .filter(|package| layer_of(&package.name).is_none())
        .map(|package| Violation::Unclassified {
            krate: package.name.clone(),
        });
    let forbidden = members.iter().flat_map(package_violations);
    unclassified.chain(forbidden).collect()
}

fn parse_dependency(dep: &Value) -> Result<Dependency, String> {
    let name = dep
        .get("name")
        .and_then(Value::as_str)
        .ok_or("a dependency has no name")?
        .to_owned();
    let kind = match dep.get("kind").and_then(Value::as_str) {
        None => DepKind::Normal,
        Some("build") => DepKind::Build,
        Some("dev") => DepKind::Dev,
        Some(other) => return Err(format!("unknown dependency kind {other}")),
    };
    Ok(Dependency { name, kind })
}

fn parse_package(package: &Value) -> Result<Package, String> {
    let name = package
        .get("name")
        .and_then(Value::as_str)
        .ok_or("a package has no name")?
        .to_owned();
    let deps = package
        .get("dependencies")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{name} has no dependency list"))?
        .iter()
        .map(parse_dependency)
        .collect::<Result<_, _>>()?;
    Ok(Package { name, deps })
}

/// Reads the output of `cargo metadata --format-version 1 --no-deps`.
pub fn parse_metadata(json: &str) -> Result<Vec<Package>, String> {
    let metadata: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let members = metadata
        .get("workspace_members")
        .and_then(Value::as_array)
        .ok_or("no workspace_members: is this cargo metadata?")?;
    metadata
        .get("packages")
        .and_then(Value::as_array)
        .ok_or("no packages: is this cargo metadata?")?
        .iter()
        .filter(|package| package.get("id").is_some_and(|id| members.contains(id)))
        .map(parse_package)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(name: &str, deps: &[(&str, DepKind)]) -> Package {
        Package {
            name: name.to_owned(),
            deps: deps
                .iter()
                .map(|(dep, kind)| Dependency {
                    name: (*dep).to_owned(),
                    kind: *kind,
                })
                .collect(),
        }
    }

    #[test]
    fn the_client_may_use_the_core() {
        let workspace = [
            package("flappy-core", &[]),
            package("flappy-client", &[("flappy-core", DepKind::Normal)]),
        ];

        assert_eq!(check(&workspace), vec![]);
    }

    #[test]
    fn the_core_may_not_use_the_client() {
        let workspace = [
            package("flappy-core", &[("flappy-client", DepKind::Normal)]),
            package("flappy-client", &[]),
        ];

        assert_eq!(
            check(&workspace),
            vec![Violation::Forbidden {
                from: "flappy-core".to_owned(),
                to: "flappy-client".to_owned(),
            }]
        );
    }

    #[test]
    fn build_dependencies_count_like_normal_ones() {
        let workspace = [
            package("flappy-core", &[("flappy-client", DepKind::Build)]),
            package("flappy-client", &[]),
        ];

        assert_eq!(check(&workspace).len(), 1);
    }

    #[test]
    fn dev_dependencies_are_free_so_tests_may_use_anything() {
        let workspace = [
            package("flappy-core", &[("flappy-client", DepKind::Dev)]),
            package("flappy-client", &[]),
        ];

        assert_eq!(check(&workspace), vec![]);
    }

    #[test]
    fn replays_may_not_know_how_they_are_stored() {
        let workspace = [
            package("flappy-replay", &[("flappy-storage", DepKind::Normal)]),
            package("flappy-storage", &[]),
        ];

        assert_eq!(check(&workspace).len(), 1);
    }

    #[test]
    fn storage_may_use_replays_and_the_core() {
        let workspace = [
            package("flappy-core", &[]),
            package("flappy-replay", &[("flappy-core", DepKind::Normal)]),
            package(
                "flappy-storage",
                &[
                    ("flappy-core", DepKind::Normal),
                    ("flappy-replay", DepKind::Normal),
                ],
            ),
        ];

        assert_eq!(check(&workspace), vec![]);
    }

    #[test]
    fn the_client_may_use_the_web_storage_bridge_but_not_the_reverse() {
        let workspace = [
            package("flappy-web-storage", &[("flappy-client", DepKind::Normal)]),
            package("flappy-client", &[("flappy-web-storage", DepKind::Normal)]),
        ];

        assert_eq!(
            check(&workspace),
            vec![Violation::Forbidden {
                from: "flappy-web-storage".to_owned(),
                to: "flappy-client".to_owned(),
            }]
        );
    }

    #[test]
    fn no_shipped_crate_may_use_a_tool() {
        let workspace = [
            package("flappy-client", &[("xtask", DepKind::Normal)]),
            package("xtask", &[]),
        ];

        assert_eq!(check(&workspace).len(), 1);
    }

    #[test]
    fn tools_may_use_anything_shipped() {
        let workspace = [
            package("flappy-core", &[]),
            package("flappy-client", &[]),
            package(
                "flappy-bench",
                &[
                    ("flappy-core", DepKind::Normal),
                    ("flappy-client", DepKind::Normal),
                ],
            ),
        ];

        assert_eq!(check(&workspace), vec![]);
    }

    #[test]
    fn dependencies_outside_the_workspace_are_not_checked() {
        let workspace = [package("flappy-core", &[("macroquad", DepKind::Normal)])];

        assert_eq!(check(&workspace), vec![]);
    }

    #[test]
    fn violations_name_the_crates_and_where_the_rule_lives() {
        let forbidden = Violation::Forbidden {
            from: "flappy-core".to_owned(),
            to: "flappy-client".to_owned(),
        };
        let unclassified = Violation::Unclassified {
            krate: "flappy-mystery".to_owned(),
        };

        assert_eq!(
            forbidden.to_string(),
            "flappy-core may not depend on flappy-client (docs/layers.md)"
        );
        assert!(
            unclassified
                .to_string()
                .starts_with("flappy-mystery has no layer")
        );
    }

    #[test]
    fn a_workspace_crate_without_a_layer_is_reported() {
        let workspace = [package("flappy-mystery", &[])];

        assert_eq!(
            check(&workspace),
            vec![Violation::Unclassified {
                krate: "flappy-mystery".to_owned(),
            }]
        );
    }

    #[test]
    fn reads_workspace_members_and_their_dependency_kinds_from_cargo_metadata() {
        let metadata = r#"{
            "packages": [
                {"id": "core-id", "name": "flappy-core", "dependencies": []},
                {"id": "client-id", "name": "flappy-client", "dependencies": [
                    {"name": "flappy-core", "kind": null},
                    {"name": "cc", "kind": "build"},
                    {"name": "proptest", "kind": "dev"}
                ]},
                {"id": "macroquad-id", "name": "macroquad", "dependencies": []}
            ],
            "workspace_members": ["core-id", "client-id"]
        }"#;

        assert_eq!(
            parse_metadata(metadata),
            Ok(vec![
                package("flappy-core", &[]),
                package(
                    "flappy-client",
                    &[
                        ("flappy-core", DepKind::Normal),
                        ("cc", DepKind::Build),
                        ("proptest", DepKind::Dev),
                    ]
                ),
            ])
        );
    }

    #[test]
    fn rejects_metadata_that_is_not_cargo_metadata() {
        assert!(parse_metadata(r#"{"hello": 1}"#).is_err());
    }
}
