//! The supply-chain checks (ADR-0039, decisions 13 and 14): cargo-deny for the Rust
//! crates, and the licenses of the web dependencies shipped in the frontend against
//! the same allow list, read from `deny.toml`.

use std::fs;
use std::process::Command;

use serde_json::{Map, Value};

pub fn check() -> Result<(), String> {
    let status = Command::new(crate::cargo_binary())
        .args(["deny", "check"])
        .status()
        .map_err(|error| format!("cargo deny: {error} (is cargo-deny installed?)"))?;
    if !status.success() {
        return Err("supply-chain: cargo deny check failed".into());
    }
    web_licenses()
}

/// Every license expression of the production web dependencies must be satisfied by
/// the allow list.
fn web_licenses() -> Result<(), String> {
    let allowed = allow_list()?;
    let output = Command::new("pnpm")
        .args(["licenses", "list", "--json", "--prod"])
        .current_dir("web")
        .output()
        .map_err(|error| format!("pnpm licenses: {error} (is pnpm installed?)"))?;
    if !output.status.success() {
        return Err(format!(
            "pnpm licenses failed (run `pnpm install` in web/ first):\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let listing: Map<String, Value> = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("pnpm licenses: unexpected output: {error}"))?;
    let mut problems = Vec::new();
    let mut packages = 0;
    for (expression, entries) in &listing {
        let names: Vec<&str> = entries
            .as_array()
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| entry["name"].as_str())
                    .collect()
            })
            .unwrap_or_default();
        packages += names.len();
        match satisfied(expression, &allowed) {
            Ok(true) => {}
            Ok(false) => problems.push(format!("{expression}: {}", names.join(", "))),
            Err(error) => problems.push(format!("{expression} ({error}): {}", names.join(", "))),
        }
    }
    if problems.is_empty() {
        println!("supply-chain: {packages} web packages, every license allowed");
        Ok(())
    } else {
        Err(format!(
            "supply-chain: web licenses outside the allow list in deny.toml:\n  {}",
            problems.join("\n  ")
        ))
    }
}

/// `licenses.allow` of `deny.toml`.
fn allow_list() -> Result<Vec<String>, String> {
    let text = fs::read_to_string("deny.toml").map_err(|error| format!("deny.toml: {error}"))?;
    let config: toml::Table = text
        .parse()
        .map_err(|error| format!("deny.toml: {error}"))?;
    config
        .get("licenses")
        .and_then(|licenses| licenses.get("allow"))
        .and_then(toml::Value::as_array)
        .map(|allow| {
            allow
                .iter()
                .filter_map(toml::Value::as_str)
                .map(String::from)
                .collect()
        })
        .ok_or_else(|| "deny.toml: no [licenses] allow list".into())
}

/// Whether an SPDX license expression is satisfied by `allowed`: one side of an OR,
/// both sides of an AND. `X WITH Y` must be allowed as a whole.
pub fn satisfied(expression: &str, allowed: &[String]) -> Result<bool, String> {
    let spaced = expression.replace('(', " ( ").replace(')', " ) ");
    let tokens: Vec<&str> = spaced.split_whitespace().collect();
    let mut parser = Parser {
        tokens: &tokens,
        position: 0,
        allowed,
    };
    let result = parser.or()?;
    if parser.position != tokens.len() {
        return Err("not a license expression".into());
    }
    Ok(result)
}

struct Parser<'a> {
    tokens: &'a [&'a str],
    position: usize,
    allowed: &'a [String],
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&'a str> {
        self.tokens.get(self.position).copied()
    }

    fn next(&mut self) -> Option<&'a str> {
        let token = self.peek();
        self.position += 1;
        token
    }

    fn or(&mut self) -> Result<bool, String> {
        let mut result = self.and()?;
        while self.peek() == Some("OR") {
            self.position += 1;
            result |= self.and()?;
        }
        Ok(result)
    }

    fn and(&mut self) -> Result<bool, String> {
        let mut result = self.atom()?;
        while self.peek() == Some("AND") {
            self.position += 1;
            result &= self.atom()?;
        }
        Ok(result)
    }

    fn atom(&mut self) -> Result<bool, String> {
        match self.next() {
            Some("(") => {
                let result = self.or()?;
                if self.next() != Some(")") {
                    return Err("unbalanced parentheses".into());
                }
                Ok(result)
            }
            Some(token @ ("AND" | "OR" | "WITH" | ")")) => Err(format!("unexpected `{token}`")),
            Some(id) => {
                let mut license = id.to_string();
                if self.peek() == Some("WITH") {
                    self.position += 1;
                    let exception = self.next().ok_or("`WITH` without an exception")?;
                    license = format!("{license} WITH {exception}");
                }
                Ok(self.allowed.contains(&license))
            }
            None => Err("incomplete expression".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowed() -> Vec<String> {
        ["MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception"]
            .map(String::from)
            .to_vec()
    }

    #[test]
    fn a_single_allowed_license_passes() {
        assert_eq!(satisfied("MIT", &allowed()), Ok(true));
    }

    #[test]
    fn a_license_outside_the_list_fails() {
        assert_eq!(satisfied("GPL-2.0-only", &allowed()), Ok(false));
        assert_eq!(satisfied("Unknown", &allowed()), Ok(false));
    }

    #[test]
    fn or_needs_one_side() {
        assert_eq!(satisfied("GPL-2.0-only OR MIT", &allowed()), Ok(true));
        assert_eq!(satisfied("(MIT OR CC0-1.0)", &allowed()), Ok(true));
    }

    #[test]
    fn and_needs_both_sides() {
        assert_eq!(satisfied("MIT AND Apache-2.0", &allowed()), Ok(true));
        assert_eq!(satisfied("MIT AND GPL-2.0-only", &allowed()), Ok(false));
    }

    #[test]
    fn and_binds_tighter_than_or() {
        assert_eq!(
            satisfied("GPL-2.0-only AND MIT OR Apache-2.0", &allowed()),
            Ok(true)
        );
        assert_eq!(
            satisfied("GPL-2.0-only AND (MIT OR Apache-2.0)", &allowed()),
            Ok(false)
        );
    }

    #[test]
    fn with_must_be_allowed_as_a_whole() {
        assert_eq!(
            satisfied("Apache-2.0 WITH LLVM-exception", &allowed()),
            Ok(true)
        );
        assert_eq!(satisfied("MIT WITH Some-exception", &allowed()), Ok(false));
    }

    #[test]
    fn a_malformed_expression_is_an_error() {
        for expression in ["", "MIT OR", "(MIT", "MIT)", "AND MIT", "MIT WITH"] {
            assert!(satisfied(expression, &allowed()).is_err(), "{expression}");
        }
    }
}
