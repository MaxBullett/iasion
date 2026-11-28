//! `cargo xtask`: the one entry point for checks and tasks, for developers and CI
//! (ADR-0038).

mod deps;
mod dev_access;
mod migrations;
mod pins;
mod quadlets;
mod supply_chain;
mod web;

use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

/// Context crates, built and tested with their `postgres` feature off (ADR-0034,
/// decision 5; ADR-0039).
const CONTEXT_CRATES: &[&str] = &[
    "iasion-household",
    "iasion-access",
    "iasion-ledger",
    "iasion-classification",
];

const USAGE: &str = "usage: cargo xtask <command>

commands:
  check     format, clippy, dependency edges, migrations, context crates without
            `postgres`, and that crates/api/openapi.json is up to date
  test      all tests of the workspace with all features
  deps      dependency edges against the context map (ADR-0034, ADR-0035)
  migrations  migration names, and one schema per migration (ADR-0033)
  quadlets  the units in deploy/quadlets (ADR-0040)
  pins      versions pinned in several places agree (ADR-0039)
  features  build and test each context crate with `postgres` off (ADR-0034)
  openapi   regenerate crates/api/openapi.json from the code, and the web client
            from it (ADR-0036)
  supply-chain  cargo-deny, and the web licenses against its allow list (ADR-0039)
  web-check the frontend: frozen install, lint, format, types, tests, client drift
            (ADR-0043)
  dev-token [--email <email>]  print a dev Access token, by default for the setup
                 email; creates the dev key pair on first use (ADR-0038)
  install-hooks  use the versioned git hooks in .githooks (ADR-0041)
  hooks-test     run the tests of the git hooks (needs Docker or Betterleaks)";

fn main() -> ExitCode {
    // Every path below is relative to the workspace root. Without this, running from
    // a subdirectory would read the wrong files, or make a second dev key pair there.
    if let Err(error) = env::set_current_dir(workspace_root()) {
        eprintln!("xtask: cannot change to the workspace root: {error}");
        return ExitCode::FAILURE;
    }
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("check") => check(),
        Some("test") => test(),
        Some("deps") => deps(),
        Some("migrations") => migrations(),
        Some("quadlets") => quadlets(),
        Some("pins") => pins(),
        Some("features") => features(),
        Some("openapi") => openapi(false).and_then(|()| web::generate_client()),
        Some("web-check") => web::check(),
        Some("supply-chain") => supply_chain::check(),
        Some("dev-token") => dev_access::dev_token(&args[1..]),
        Some("install-hooks") => install_hooks(),
        Some("hooks-test") => run("sh", &[".githooks/test.sh"]),
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn check() -> Result<(), String> {
    cargo(&["fmt", "--all", "--check"])?;
    cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
    ])?;
    deps()?;
    migrations()?;
    quadlets()?;
    pins()?;
    features()?;
    openapi(true)
}

/// The committed API description, generated from the code (ADR-0036, decision 2).
const OPENAPI: &str = "crates/api/openapi.json";

/// Writes the API description, or with `check` fails if the committed one differs.
fn openapi(check: bool) -> Result<(), String> {
    let output = Command::new(cargo_binary())
        .args([
            "run",
            "--quiet",
            "--package",
            "iasion-api",
            "--example",
            "openapi",
        ])
        .output()
        .map_err(|error| format!("openapi: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "openapi: generating failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let generated = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    if !check {
        fs::write(OPENAPI, &generated).map_err(|error| format!("{OPENAPI}: {error}"))?;
        println!("openapi: wrote {OPENAPI}");
        return Ok(());
    }
    let committed = fs::read_to_string(OPENAPI).unwrap_or_default();
    if committed == generated {
        println!("openapi: {OPENAPI} matches the code");
        Ok(())
    } else {
        Err(format!(
            "openapi: {OPENAPI} differs from the code; run `cargo xtask openapi` and commit it"
        ))
    }
}

fn test() -> Result<(), String> {
    cargo(&["test", "--workspace", "--all-features"])
}

fn deps() -> Result<(), String> {
    let output = Command::new(cargo_binary())
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|error| format!("cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let json = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    let found = deps::violations(&deps::packages_from_metadata(&json)?);
    if found.is_empty() {
        println!("deps: all dependency edges follow the context map");
        Ok(())
    } else {
        Err(format!("deps:\n  {}", found.join("\n  ")))
    }
}

fn migrations() -> Result<(), String> {
    let entries = fs::read_dir(migrations::DIRECTORY)
        .map_err(|error| format!("migrations: {}: {error}", migrations::DIRECTORY))?;
    let mut found = Vec::new();
    let mut count = 0;
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let sql = fs::read_to_string(&path).map_err(|error| format!("{name}: {error}"))?;
        found.extend(migrations::problems(&name, &sql));
        count += 1;
    }
    if found.is_empty() {
        println!("migrations: {count} migrations, each in its own schema");
        Ok(())
    } else {
        Err(format!("migrations:\n  {}", found.join("\n  ")))
    }
}

fn quadlets() -> Result<(), String> {
    let entries = fs::read_dir(quadlets::DIRECTORY)
        .map_err(|error| format!("quadlets: {}: {error}", quadlets::DIRECTORY))?;
    let mut found = Vec::new();
    let mut count = 0;
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path
            .extension()
            .is_none_or(|extension| extension != "container")
        {
            continue;
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let text = fs::read_to_string(&path).map_err(|error| format!("{name}: {error}"))?;
        found.extend(quadlets::problems(&name, &text));
        count += 1;
    }
    if found.is_empty() {
        println!("quadlets: {count} containers, health checks run without a shell");
        Ok(())
    } else {
        Err(format!("quadlets:\n  {}", found.join("\n  ")))
    }
}

fn pins() -> Result<(), String> {
    let found = pins::problems(|file| fs::read_to_string(file).map_err(|e| e.to_string()));
    if found.is_empty() {
        println!(
            "pins: {} tools, each pinned to one version everywhere",
            pins::PINS.len()
        );
        Ok(())
    } else {
        Err(format!("pins:\n  {}", found.join("\n  ")))
    }
}

fn features() -> Result<(), String> {
    for name in CONTEXT_CRATES {
        cargo(&[
            "check",
            "--package",
            name,
            "--no-default-features",
            "--all-targets",
        ])?;
        cargo(&["test", "--package", name, "--no-default-features"])?;
    }
    println!("features: every context crate builds and tests without `postgres`");
    Ok(())
}

fn install_hooks() -> Result<(), String> {
    run("git", &["config", "core.hooksPath", ".githooks"])?;
    println!("install-hooks: git now uses the hooks in .githooks");
    Ok(())
}

fn cargo(args: &[&str]) -> Result<(), String> {
    run(&cargo_binary(), args)
}

fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|error| format!("{program} {}: {error}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("failed: {program} {}", args.join(" ")))
    }
}

/// The workspace root: the parent of this crate's directory.
fn workspace_root() -> &'static Path {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap_or(manifest_dir)
}

/// The cargo that runs `xtask`, so the pinned toolchain is used throughout.
pub fn cargo_binary() -> String {
    env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_workspace_root_holds_the_workspace_manifest() {
        let manifest = fs::read_to_string(workspace_root().join("Cargo.toml")).unwrap();
        assert!(manifest.contains("[workspace]"));
    }
}
