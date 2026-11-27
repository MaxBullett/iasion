//! Versions pinned in more than one place must agree. Dependabot updates one place at
//! a time (ADR-0039, decision 16), and some places it cannot see at all, so a bump
//! that misses its twins fails here instead of drifting.

/// One pinned tool and every place that names its version: a file, and the text right
/// before the version there. Every occurrence in the file counts.
pub struct Pin {
    pub name: &'static str,
    pub places: &'static [(&'static str, &'static str)],
    /// A checksum, not a version: any hex string, not only ones starting with a digit.
    pub hex: bool,
}

pub const PINS: &[Pin] = &[
    Pin {
        name: "Rust",
        places: &[
            ("rust-toolchain.toml", "channel = \""),
            ("Dockerfile", "FROM rust:"),
            ("Dockerfile", "RUSTUP_TOOLCHAIN="),
            (".devcontainer/Dockerfile", "FROM rust:"),
        ],
        hex: false,
    },
    Pin {
        name: "Node",
        places: &[
            (".node-version", ""),
            ("Dockerfile", "node:"),
            (".devcontainer/Dockerfile", "node:"),
        ],
        hex: false,
    },
    Pin {
        name: "pnpm",
        places: &[
            ("web/package.json", "\"packageManager\": \"pnpm@"),
            ("Dockerfile", "PNPM_VERSION="),
            (".devcontainer/Dockerfile", "PNPM_VERSION="),
        ],
        hex: false,
    },
    Pin {
        name: "Playwright",
        places: &[
            ("web/package.json", "\"@playwright/test\": \""),
            (".devcontainer/Dockerfile", "PLAYWRIGHT_VERSION="),
        ],
        hex: false,
    },
    Pin {
        name: "cargo-deny",
        places: &[
            (".devcontainer/Dockerfile", "CARGO_DENY_VERSION="),
            (".github/workflows/ci.yml", "CARGO_DENY_VERSION: "),
        ],
        hex: false,
    },
    Pin {
        name: "cargo-deny's x86_64 checksum",
        places: &[
            (".devcontainer/Dockerfile", "arch=x86_64; sum="),
            (".github/workflows/ci.yml", "CARGO_DENY_SHA256: "),
        ],
        hex: true,
    },
    Pin {
        name: "Betterleaks",
        places: &[
            (".githooks/lib.sh", "BETTERLEAKS_VERSION="),
            (".devcontainer/Dockerfile", "BETTERLEAKS_VERSION="),
        ],
        hex: false,
    },
    Pin {
        name: "PostgreSQL",
        places: &[
            ("compose.yaml", "postgres:"),
            (".github/workflows/ci.yml", "postgres:"),
            ("deploy/quadlets/iasion-db.container", "postgres:"),
            ("deploy/install.sh", "postgres:"),
        ],
        hex: false,
    },
    Pin {
        name: "busybox (access-dev)",
        places: &[
            ("compose.yaml", "busybox:"),
            (".github/workflows/ci.yml", "busybox:"),
        ],
        hex: false,
    },
    Pin {
        name: "restic",
        places: &[
            ("deploy/bin/iasion-backup", "restic/restic:"),
            ("deploy/install.sh", "restic/restic:"),
        ],
        hex: false,
    },
    Pin {
        name: "cloudflared",
        places: &[
            (
                "deploy/quadlets/iasion-cloudflared.container",
                "cloudflared:",
            ),
            ("deploy/install.sh", "cloudflared:"),
        ],
        hex: false,
    },
];

/// The versions after every occurrence of `prefix` in `text`. A version is the run of
/// letters, digits and dots that follows, and must start with a digit (any hex digit
/// for a checksum), so that e.g. `postgres://` in a URL is no pin.
pub fn versions(text: &str, prefix: &str, hex: bool) -> Vec<String> {
    let take = |rest: &str| -> Option<String> {
        let value: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '.')
            .collect();
        let value = value.trim_end_matches('.').to_string();
        let first = value.chars().next()?;
        let starts_right = if hex {
            first.is_ascii_hexdigit()
        } else {
            first.is_ascii_digit()
        };
        starts_right.then_some(value)
    };
    if prefix.is_empty() {
        return take(text.trim()).into_iter().collect();
    }
    text.match_indices(prefix)
        .filter_map(|(at, _)| take(&text[at + prefix.len()..]))
        .collect()
}

/// What disagrees, given a way to read a file.
pub fn problems(read: impl Fn(&str) -> Result<String, String>) -> Vec<String> {
    let mut found = Vec::new();
    for pin in PINS {
        let mut seen: Vec<(String, &str)> = Vec::new();
        for (file, prefix) in pin.places {
            let text = match read(file) {
                Ok(text) => text,
                Err(error) => {
                    found.push(format!("{}: {file}: {error}", pin.name));
                    continue;
                }
            };
            let values = versions(&text, prefix, pin.hex);
            if values.is_empty() {
                found.push(format!(
                    "{}: {file} no longer names it after `{prefix}`; update xtask/src/pins.rs",
                    pin.name
                ));
            }
            for value in values {
                seen.push((value, file));
            }
        }
        let mut distinct: Vec<&String> = seen.iter().map(|(value, _)| value).collect();
        distinct.sort();
        distinct.dedup();
        if distinct.len() > 1 {
            let places: Vec<String> = seen
                .iter()
                .map(|(value, file)| format!("{file} {value}"))
                .collect();
            found.push(format!("{} differs: {}", pin.name, places.join(", ")));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_version_ends_where_its_characters_end() {
        assert_eq!(
            versions("FROM rust:1.100.0 AS server", "FROM rust:", false),
            ["1.100.0"]
        );
        assert_eq!(
            versions("node:26.13.1-trixie-slim", "node:", false),
            ["26.13.1"]
        );
        assert_eq!(versions("\"pnpm@12.23.0\",", "\"pnpm@", false), ["12.23.0"]);
    }

    #[test]
    fn every_occurrence_counts() {
        let text = "image: postgres:18.7\n  image: postgres:18.6\n";
        assert_eq!(versions(text, "postgres:", false), ["18.7", "18.6"]);
    }

    #[test]
    fn a_url_is_not_a_pin() {
        let text = "DATABASE_URL: postgres://postgres@localhost\nimage: postgres:18.7";
        assert_eq!(versions(text, "postgres:", false), ["18.7"]);
    }

    #[test]
    fn a_whole_file_can_be_the_version() {
        assert_eq!(versions("26.13.1\n", "", false), ["26.13.1"]);
    }

    #[test]
    fn a_checksum_may_start_with_a_letter() {
        assert_eq!(versions("sum=ab12;", "sum=", true), ["ab12"]);
        assert_eq!(versions("sum=ab12;", "sum=", false), Vec::<String>::new());
    }

    #[test]
    fn the_repository_agrees_with_itself() {
        // Run from the workspace root, as `cargo xtask` does.
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
        let found = problems(|file| {
            std::fs::read_to_string(format!("{root}/{file}")).map_err(|e| e.to_string())
        });
        assert_eq!(found, Vec::<String>::new());
    }
}
