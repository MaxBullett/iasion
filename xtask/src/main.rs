//! `cargo xtask`: the one entry point for checks and tasks, for developers and CI
//! (ADR-0038).

mod deps;

use std::env;
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
  check     format, clippy, dependency edges, context crates without `postgres`
  test      all tests of the workspace with all features
  deps      dependency edges against the context map (ADR-0034, ADR-0035)
  features  build and test each context crate with `postgres` off (ADR-0034)";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("check") => check(),
        Some("test") => test(),
        Some("deps") => deps(),
        Some("features") => features(),
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
    features()
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

fn cargo(args: &[&str]) -> Result<(), String> {
    let status = Command::new(cargo_binary())
        .args(args)
        .status()
        .map_err(|error| format!("cargo {}: {error}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("failed: cargo {}", args.join(" ")))
    }
}

/// The cargo that runs `xtask`, so the pinned toolchain is used throughout.
fn cargo_binary() -> String {
    env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}
