//! Prints the API description; `cargo xtask openapi` writes it to
//! `crates/api/openapi.json`, and `cargo xtask check` compares (ADR-0036, decision 2).

fn main() -> std::process::ExitCode {
    match iasion_api::openapi().to_pretty_json() {
        Ok(json) => {
            println!("{json}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("openapi: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
