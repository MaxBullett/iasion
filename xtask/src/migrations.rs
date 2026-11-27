//! Checks the migration files: their names, and that each one touches only its own
//! context's schema (ADR-0033, decision 15).

/// The contexts that own a schema (ADR-0033, decision 2).
pub const SCHEMAS: &[&str] = &["household", "access", "ledger", "classification"];

pub const DIRECTORY: &str = "crates/migrations/migrations";

/// Returns one message per problem with the migration `file_name` and its `sql`.
pub fn problems(file_name: &str, sql: &str) -> Vec<String> {
    let Some(context) = context_of(file_name) else {
        return vec![format!(
            "{file_name}: name must be <14-digit timestamp>_<context>_<name>.sql with a \
             context of {} (ADR-0033, decision 15)",
            SCHEMAS.join(", ")
        )];
    };
    let words = words(&strip_comments(sql));
    let mut found = Vec::new();
    found.extend(contract_problems(file_name, &words));
    for other in SCHEMAS.iter().filter(|schema| **schema != context) {
        let qualified = words
            .iter()
            .any(|word| word.split('.').next() == Some(*other) && word.contains('.'));
        let as_schema = words
            .windows(2)
            .any(|pair| pair[0] == "schema" && pair[1] == *other);
        if qualified || as_schema {
            found.push(format!(
                "{file_name}: a {context} migration names the {other} schema (ADR-0033, \
                 decision 15)"
            ));
        }
    }
    found
}

/// A migration that removes something is a contract migration (ADR-0040, decision
/// 11). Before one runs, `migrate` must take a backup that is kept and prove it
/// restores; until `iasion migrate` does that, contract migrations are refused, so that
/// none runs without it.
fn contract_problems(file_name: &str, words: &[String]) -> Vec<String> {
    const NOT_YET: &str = "contract migrations need the kept, restore-checked backup of \
        ADR-0040 decision 11, which `iasion migrate` does not take yet";
    let named = file_name
        .strip_suffix(".sql")
        .is_some_and(|stem| stem.split('_').any(|part| part == "contract"));
    let removes = words
        .iter()
        .enumerate()
        .any(|(i, word)| match word.as_str() {
            "drop" | "truncate" => true,
            // `ON DELETE` in a foreign key removes nothing by itself.
            "delete" => i == 0 || words[i - 1] != "on",
            _ => false,
        });
    match (named, removes) {
        (true, _) => vec![format!("{file_name}: {NOT_YET}")],
        (false, true) => vec![format!(
            "{file_name}: removes something (DROP, TRUNCATE or DELETE), so it is a contract \
             migration; {NOT_YET}"
        )],
        (false, false) => Vec::new(),
    }
}

/// The context in a name `<timestamp>_<context>_<name>.sql`, if the name has that form.
fn context_of(file_name: &str) -> Option<&'static str> {
    let stem = file_name.strip_suffix(".sql")?;
    let (timestamp, rest) = stem.split_once('_')?;
    if timestamp.len() != 14 || !timestamp.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let context = SCHEMAS.iter().find(|schema| {
        rest.strip_prefix(**schema)
            .and_then(|r| r.strip_prefix('_'))
            .is_some()
    })?;
    let name = &rest[context.len() + 1..];
    let valid = !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    valid.then_some(*context)
}

/// The SQL without `--` line comments and `/* */` block comments.
fn strip_comments(sql: &str) -> String {
    let mut out = String::with_capacity(sql.len());
    let mut rest = sql;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("--") {
            rest = after.split_once('\n').map_or("", |(_, tail)| tail);
            out.push('\n');
        } else if let Some(after) = rest.strip_prefix("/*") {
            rest = after.split_once("*/").map_or("", |(_, tail)| tail);
            out.push(' ');
        } else {
            let mut chars = rest.chars();
            if let Some(c) = chars.next() {
                out.push(c);
            }
            rest = chars.as_str();
        }
    }
    out
}

/// Lower-case words of identifier characters and dots, with double quotes removed.
fn words(sql: &str) -> Vec<String> {
    sql.to_lowercase()
        .replace('"', "")
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
        .filter(|word| !word.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAME: &str = "20261126000003_ledger_create_accounts.sql";

    #[test]
    fn a_migration_in_its_own_schema_passes() {
        let sql = "CREATE TABLE ledger.accounts (id uuid);\n-- not household.parties\n";
        assert_eq!(problems(NAME, sql), Vec::<String>::new());
    }

    #[test]
    fn naming_another_schema_fails() {
        let sql = "CREATE TABLE ledger.accounts (owner uuid REFERENCES household.parties);";
        assert_eq!(problems(NAME, sql).len(), 1);
    }

    #[test]
    fn a_quoted_other_schema_fails() {
        assert_eq!(
            problems(NAME, r#"SELECT 1 FROM "Household".parties"#).len(),
            1
        );
    }

    #[test]
    fn creating_another_schema_fails() {
        assert_eq!(problems(NAME, "CREATE SCHEMA access;").len(), 1);
    }

    #[test]
    fn a_column_named_like_a_schema_passes() {
        assert_eq!(
            problems(NAME, "ALTER TABLE ledger.accounts ADD access text;"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_contract_migration_is_refused_for_now() {
        let name = "20261126000009_ledger_contract_drop_old_column.sql";
        assert_eq!(problems(name, "SELECT 1;").len(), 1);
    }

    #[test]
    fn removing_something_makes_it_a_contract_migration() {
        for sql in [
            "ALTER TABLE ledger.accounts DROP COLUMN name;",
            "DROP TABLE ledger.accounts;",
            "TRUNCATE ledger.accounts;",
            "DELETE FROM ledger.accounts;",
        ] {
            assert_eq!(problems(NAME, sql).len(), 1, "{sql}");
        }
    }

    #[test]
    fn a_foreign_key_on_delete_removes_nothing() {
        let sql = "CREATE TABLE ledger.flows (account uuid REFERENCES ledger.accounts \
                   ON DELETE RESTRICT);";
        assert_eq!(problems(NAME, sql), Vec::<String>::new());
    }

    #[test]
    fn a_word_in_a_comment_does_not_count() {
        assert_eq!(
            problems(NAME, "-- we never drop anything\nSELECT 1;"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn names_must_follow_the_pattern() {
        for bad in [
            "20261126_ledger_x.sql",
            "20261126000003_reports_x.sql",
            "20261126000003_ledger_.sql",
            "20261126000003_ledger_Create.sql",
            "20261126000003_ledgerx.sql",
            "20261126000003_ledger_x.txt",
        ] {
            assert_eq!(problems(bad, "").len(), 1, "{bad} should be rejected");
        }
    }
}
