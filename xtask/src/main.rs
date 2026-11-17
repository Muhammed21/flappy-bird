//! `cargo xtask ci`     fmt, clippy (desktop + wasm), tests, layers, cargo-deny
//! `cargo xtask layers` checks docs/layers.md
//! `cargo xtask wasm`   builds the browser version into dist/

mod layers;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde_json::Value;

type Task = Result<(), String>;

const WASM_TARGET: &str = "wasm32-unknown-unknown";
const DIST: &str = "dist";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["ci"] => ci(),
        ["layers"] => check_layers(),
        ["wasm"] => wasm(),
        _ => Err("usage: cargo xtask <ci|layers|wasm>".to_owned()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

fn cargo(args: &[&str]) -> Task {
    eprintln!("$ cargo {}", args.join(" "));
    let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()))
        .args(args)
        .status()
        .map_err(|error| format!("cargo {}: {error}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {} failed", args.join(" ")))
    }
}

fn cargo_output(args: &[&str]) -> Result<String, String> {
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned()))
        .args(args)
        .output()
        .map_err(|error| format!("cargo {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    String::from_utf8(output.stdout).map_err(|error| error.to_string())
}

fn ci() -> Task {
    cargo(&["fmt", "--all", "--", "--check"])?;
    cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--locked",
        "--",
        "-D",
        "warnings",
    ])?;
    cargo(&[
        "clippy",
        "-p",
        "flappy-core",
        "-p",
        "flappy-client",
        "--target",
        WASM_TARGET,
        "--locked",
        "--",
        "-D",
        "warnings",
    ])?;
    cargo(&["test", "--workspace", "--locked"])?;
    check_layers()?;
    deny()
}

fn deny() -> Task {
    let installed = Command::new("cargo-deny").arg("--version").output().is_ok();
    match (installed, std::env::var_os("CI").is_some()) {
        (true, _) => cargo(&["deny", "--locked", "check"]),
        (false, true) => Err("cargo-deny is required in CI".to_owned()),
        (false, false) => {
            eprintln!(
                "xtask: cargo-deny not installed, skipped (cargo install --locked cargo-deny)"
            );
            Ok(())
        }
    }
}

fn check_layers() -> Task {
    let metadata = cargo_output(&["metadata", "--format-version", "1", "--no-deps", "--locked"])?;
    let violations = layers::check(&layers::parse_metadata(&metadata)?);
    if violations.is_empty() {
        eprintln!("xtask: layers ok");
        return Ok(());
    }
    for violation in &violations {
        eprintln!("  {violation}");
    }
    Err(format!("{} layer violation(s)", violations.len()))
}

fn miniquad_js() -> Result<PathBuf, String> {
    let metadata: Value = serde_json::from_str(&cargo_output(&[
        "metadata",
        "--format-version",
        "1",
        "--locked",
        "--filter-platform",
        WASM_TARGET,
    ])?)
    .map_err(|error| error.to_string())?;
    let manifest = metadata
        .get("packages")
        .and_then(Value::as_array)
        .and_then(|packages| {
            packages
                .iter()
                .find(|package| package.get("name").and_then(Value::as_str) == Some("miniquad"))
        })
        .and_then(|package| package.get("manifest_path").and_then(Value::as_str))
        .ok_or("miniquad is not in the dependency graph")?;
    Path::new(manifest)
        .parent()
        .map(|dir| dir.join("js").join("gl.js"))
        .ok_or_else(|| format!("no folder for {manifest}"))
}

fn copy(from: &Path, to: &Path) -> Task {
    std::fs::copy(from, to)
        .map(|_| ())
        .map_err(|error| format!("copy {} -> {}: {error}", from.display(), to.display()))
}

fn wasm() -> Task {
    cargo(&[
        "build",
        "-p",
        "flappy-client",
        "--target",
        WASM_TARGET,
        "--release",
        "--locked",
    ])?;
    let dist = Path::new(DIST);
    std::fs::create_dir_all(dist).map_err(|error| format!("create {DIST}: {error}"))?;
    copy(
        &Path::new("target")
            .join(WASM_TARGET)
            .join("release")
            .join("flappy-client.wasm"),
        &dist.join("flappy-client.wasm"),
    )?;
    copy(&miniquad_js()?, &dist.join("gl.js"))?;
    copy(Path::new("web/index.html"), &dist.join("index.html"))?;
    copy(Path::new("web/boot.js"), &dist.join("boot.js"))?;
    eprintln!(
        "xtask: {DIST}/ ready, serve it with any static server (e.g. python3 -m http.server -d {DIST})"
    );
    Ok(())
}
