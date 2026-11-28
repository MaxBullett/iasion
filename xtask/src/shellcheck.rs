//! Every shell script of the repository goes through shellcheck (ADR-0044, decision 9):
//! `./dev`, the git hooks, and the scripts in `deploy/`.

/// Whether a tracked file is a shell script, by its name or its first line. Sourced
/// files have no `#!` line, so `.sh` counts too.
pub fn is_shell_script(path: &str, first_line: &str) -> bool {
    if path.ends_with(".sh") {
        return true;
    }
    let Some(interpreter) = first_line.strip_prefix("#!") else {
        return false;
    };
    let mut words = interpreter.split_whitespace();
    let program = match words.next() {
        Some(env) if env.ends_with("/env") => words.next(),
        other => other,
    };
    program
        .is_some_and(|program| matches!(program.rsplit('/').next(), Some("sh" | "bash" | "dash")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_script_is_known_by_its_interpreter() {
        assert!(is_shell_script("dev", "#!/bin/sh"));
        assert!(is_shell_script("deploy/bin/iasion-backup", "#!/bin/sh"));
        assert!(is_shell_script("tool", "#!/usr/bin/env bash"));
        assert!(is_shell_script("tool", "#!/bin/bash -e"));
    }

    #[test]
    fn a_sourced_file_is_known_by_its_name() {
        assert!(is_shell_script(".githooks/lib.sh", "# Shared code"));
    }

    #[test]
    fn other_files_are_not_scripts() {
        assert!(!is_shell_script("README.md", "# Iasion"));
        assert!(!is_shell_script("tool", "#!/usr/bin/env node"));
        assert!(!is_shell_script("tool", "#!/usr/bin/python3"));
        assert!(!is_shell_script("crates/api/src/lib.rs", "//! The API"));
        assert!(!is_shell_script("shell.txt", "#!"));
    }
}
