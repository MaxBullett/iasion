//! Checks of the units in `deploy/quadlets/` (ADR-0040).

/// Where the quadlets are.
pub const DIRECTORY: &str = "deploy/quadlets";

/// What is wrong with the quadlet `file_name`. A container of the app's image (written
/// as `@IASION_IMAGE@`) must give `HealthCmd` in the array form: Podman runs a plain
/// command through `/bin/sh`, which the distroless image does not have, so every check
/// would fail and `HealthOnFailure=kill` would restart the app forever.
pub fn problems(file_name: &str, text: &str) -> Vec<String> {
    let value = |key: &str| {
        text.lines()
            .map(str::trim)
            .filter_map(|line| line.strip_prefix(key)?.strip_prefix('='))
            .map(str::trim)
            .next_back()
    };
    let ours = value("Image") == Some("@IASION_IMAGE@");
    match value("HealthCmd") {
        Some(command) if ours && !command.starts_with('[') => vec![format!(
            "{file_name}: HealthCmd must be a JSON array, e.g. HealthCmd=[\"/iasion\", \"health\"]: \
             the image has no shell to run `{command}` (ADR-0040)"
        )],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_array_form_passes() {
        let text = "[Container]\nImage=@IASION_IMAGE@\nHealthCmd=[\"/iasion\", \"health\"]\n";
        assert_eq!(problems("iasion-app.container", text), Vec::<String>::new());
    }

    #[test]
    fn a_plain_command_for_our_image_fails() {
        let text = "[Container]\nImage=@IASION_IMAGE@\nHealthCmd=/iasion health\n";
        assert_eq!(problems("iasion-app.container", text).len(), 1);
    }

    #[test]
    fn a_plain_command_for_an_image_with_a_shell_passes() {
        let text = "[Container]\nImage=docker.io/library/postgres:18.7\n\
                    HealthCmd=pg_isready --username postgres\n";
        assert_eq!(problems("iasion-db.container", text), Vec::<String>::new());
    }

    #[test]
    fn a_container_without_a_health_check_passes() {
        let text = "[Container]\nImage=@IASION_IMAGE@\nExec=migrate\n";
        assert_eq!(
            problems("iasion-migrate.container", text),
            Vec::<String>::new()
        );
    }
}
