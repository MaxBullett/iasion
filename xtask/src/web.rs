//! The frontend's checks and the client generated from the API description
//! (ADR-0036, ADR-0043), run through pnpm in `web/`.

use std::fs;
use std::process::Command;

const WEB: &str = "web";
const DESCRIPTION: &str = "../crates/api/openapi.json";
const CLIENT: &str = "web/src/api/generated/schema.ts";

/// Regenerates the client types from the committed API description.
pub fn generate_client() -> Result<(), String> {
    pnpm(&[
        "exec",
        "openapi-typescript",
        DESCRIPTION,
        "--output",
        "src/api/generated/schema.ts",
    ])?;
    println!("client: wrote {CLIENT}");
    Ok(())
}

/// What the `web` job runs: install with the frozen lockfile, lint, format, types,
/// tests, and the client drift check (ADR-0043, decision 16).
pub fn check() -> Result<(), String> {
    pnpm(&["install", "--frozen-lockfile"])?;
    pnpm(&["run", "lint"])?;
    pnpm(&["run", "format:check"])?;
    pnpm(&["run", "typecheck"])?;
    pnpm(&["run", "test"])?;
    client_drift()
}

/// Fails when the committed client types differ from what the description generates.
fn client_drift() -> Result<(), String> {
    let generated = "node_modules/.tmp/schema.ts";
    pnpm(&[
        "exec",
        "openapi-typescript",
        DESCRIPTION,
        "--output",
        generated,
    ])?;
    let generated = fs::read_to_string(format!("{WEB}/{generated}"))
        .map_err(|error| format!("client: {error}"))?;
    let committed = fs::read_to_string(CLIENT).unwrap_or_default();
    if generated == committed {
        println!("client: {CLIENT} matches the API description");
        Ok(())
    } else {
        Err(format!(
            "client: {CLIENT} differs from the API description; run `cargo xtask openapi` and commit it"
        ))
    }
}

fn pnpm(args: &[&str]) -> Result<(), String> {
    let status = Command::new("pnpm")
        .args(args)
        .current_dir(WEB)
        .status()
        .map_err(|error| format!("pnpm {}: {error} (is pnpm installed?)", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("failed: pnpm {}", args.join(" ")))
    }
}
