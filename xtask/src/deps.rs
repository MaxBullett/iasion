//! Dependency edges between workspace crates, checked against the context map
//! (ADR-0034, decision 7) and against crates that only some workspace crates may use
//! (ADR-0035, decision 3).

/// Workspace crates and the workspace crates each may depend on, without the `iasion-`
/// prefix (ADR-0034, decision 7).
pub const ALLOWED: &[(&str, &[&str])] = &[
    ("shared-kernel", &[]),
    ("household", &["shared-kernel"]),
    ("access", &["household", "shared-kernel"]),
    ("ledger", &["household", "shared-kernel"]),
    ("classification", &["ledger", "shared-kernel"]),
    (
        "views",
        &["household", "ledger", "classification", "shared-kernel"],
    ),
    (
        "app",
        &[
            "household",
            "access",
            "ledger",
            "classification",
            "views",
            "shared-kernel",
        ],
    ),
    ("api", &["app", "views", "shared-kernel"]),
    (
        "server",
        &[
            "household",
            "access",
            "ledger",
            "classification",
            "views",
            "app",
            "api",
            "shared-kernel",
            "migrations",
        ],
    ),
    ("migrations", &[]),
];

/// External crates that only the listed workspace crates may depend on.
pub const RESTRICTED: &[(&str, &[&str])] = &[("axum", &["api", "server"])];

/// Packages that are tooling, not part of the application.
const TOOLING: &[&str] = &["xtask"];

const PREFIX: &str = "iasion-";

/// One workspace package and the names of the packages it depends on, of any kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub dependencies: Vec<String>,
}

/// Returns one message per edge that the context map or the restrictions do not allow.
pub fn violations(packages: &[Package]) -> Vec<String> {
    let mut found = Vec::new();
    for package in packages {
        if TOOLING.contains(&package.name.as_str()) {
            continue;
        }
        let Some(name) = package.name.strip_prefix(PREFIX) else {
            found.push(format!(
                "{}: workspace crates are named `{PREFIX}<name>` (ADR-0034)",
                package.name
            ));
            continue;
        };
        let Some((_, allowed)) = ALLOWED.iter().find(|(crate_name, _)| *crate_name == name) else {
            found.push(format!(
                "{}: not in the context map's list of crates (ADR-0034, decision 7)",
                package.name
            ));
            continue;
        };
        for dependency in &package.dependencies {
            if let Some(target) = dependency.strip_prefix(PREFIX) {
                if !allowed.contains(&target) {
                    found.push(format!(
                        "{} may not depend on {dependency} (ADR-0034, decision 7)",
                        package.name
                    ));
                }
            } else if let Some((_, users)) = RESTRICTED
                .iter()
                .find(|(external, _)| *external == dependency.as_str())
                && !users.contains(&name)
            {
                found.push(format!(
                    "{} may not depend on {dependency}; only {} may (ADR-0035)",
                    package.name,
                    users.join(", ")
                ));
            }
        }
    }
    found
}

/// Reads the workspace packages from the JSON of `cargo metadata --no-deps`.
pub fn packages_from_metadata(json: &str) -> Result<Vec<Package>, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|error| format!("cargo metadata: {error}"))?;
    let packages = value["packages"]
        .as_array()
        .ok_or("cargo metadata: no `packages` array")?;
    packages
        .iter()
        .map(|package| {
            let name = package["name"]
                .as_str()
                .ok_or("cargo metadata: package without a name")?
                .to_string();
            let dependencies = package["dependencies"]
                .as_array()
                .ok_or("cargo metadata: package without `dependencies`")?
                .iter()
                .map(|dependency| {
                    dependency["name"]
                        .as_str()
                        .map(str::to_string)
                        .ok_or_else(|| "cargo metadata: dependency without a name".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Package { name, dependencies })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(name: &str, dependencies: &[&str]) -> Package {
        Package {
            name: name.to_string(),
            dependencies: dependencies.iter().map(|d| d.to_string()).collect(),
        }
    }

    #[test]
    fn edges_along_the_context_map_pass() {
        let packages = [
            package(
                "iasion-ledger",
                &["iasion-household", "iasion-shared-kernel"],
            ),
            package("iasion-api", &["iasion-app", "axum", "serde_json"]),
            package("xtask", &["iasion-ledger"]),
        ];
        assert_eq!(violations(&packages), Vec::<String>::new());
    }

    #[test]
    fn an_upstream_crate_depending_on_a_downstream_one_fails() {
        let found = violations(&[package("iasion-household", &["iasion-ledger"])]);
        assert_eq!(
            found,
            ["iasion-household may not depend on iasion-ledger (ADR-0034, decision 7)"]
        );
    }

    #[test]
    fn classification_may_not_depend_on_household() {
        assert_eq!(
            violations(&[package("iasion-classification", &["iasion-household"])]).len(),
            1
        );
    }

    #[test]
    fn axum_outside_api_and_server_fails() {
        let found = violations(&[package("iasion-ledger", &["axum"])]);
        assert_eq!(
            found,
            ["iasion-ledger may not depend on axum; only api, server may (ADR-0035)"]
        );
    }

    #[test]
    fn a_crate_missing_from_the_map_fails() {
        assert_eq!(violations(&[package("iasion-reports", &[])]).len(), 1);
    }

    #[test]
    fn a_crate_without_the_prefix_fails() {
        assert_eq!(violations(&[package("ledger", &[])]).len(), 1);
    }

    #[test]
    fn every_allowed_target_is_itself_in_the_map() {
        for (name, targets) in ALLOWED {
            for target in *targets {
                assert!(
                    ALLOWED.iter().any(|(other, _)| other == target),
                    "{name} lists {target}, which is not in the map"
                );
            }
        }
    }

    #[test]
    fn metadata_json_is_read() {
        let json = r#"{"packages": [
            {"name": "iasion-ledger", "dependencies": [
                {"name": "iasion-household", "kind": null},
                {"name": "proptest", "kind": "dev"}
            ]}
        ]}"#;
        assert_eq!(
            packages_from_metadata(json),
            Ok(vec![package(
                "iasion-ledger",
                &["iasion-household", "proptest"]
            )])
        );
    }
}
