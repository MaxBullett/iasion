//! Applies all migrations to a fresh database (ADR-0039: migrations on an empty
//! database). Needs `DATABASE_URL` with a superuser and the two roles created by
//! `deploy/postgres/init-roles.sh`.

use sqlx::PgPool;

const SCHEMAS: [&str; 4] = ["household", "access", "ledger", "classification"];

#[sqlx::test(migrator = "iasion_migrations::MIGRATOR")]
async fn every_context_has_its_schema(pool: PgPool) -> sqlx::Result<()> {
    for schema in SCHEMAS {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = $1)")
                .bind(schema)
                .fetch_one(&pool)
                .await?;
        assert!(exists, "schema {schema} is missing");
    }
    Ok(())
}

#[sqlx::test(migrator = "iasion_migrations::MIGRATOR")]
async fn the_application_role_may_use_every_schema(pool: PgPool) -> sqlx::Result<()> {
    for schema in SCHEMAS {
        let usage: bool =
            sqlx::query_scalar("SELECT has_schema_privilege('iasion_app', $1, 'USAGE')")
                .bind(schema)
                .fetch_one(&pool)
                .await?;
        assert!(usage, "iasion_app has no USAGE on {schema}");
    }
    Ok(())
}

#[sqlx::test(migrator = "iasion_migrations::MIGRATOR")]
async fn the_application_role_may_not_update_or_delete(pool: PgPool) -> sqlx::Result<()> {
    // A table created after the migrations, as later migrations will: the default
    // privileges give the application role SELECT and INSERT only (ADR-0033, decision 7).
    sqlx::query("CREATE TABLE ledger.probe (id int)")
        .execute(&pool)
        .await?;
    for (privilege, expected) in [
        ("SELECT", true),
        ("INSERT", true),
        ("UPDATE", false),
        ("DELETE", false),
    ] {
        let granted: bool =
            sqlx::query_scalar("SELECT has_table_privilege('iasion_app', 'ledger.probe', $1)")
                .bind(privilege)
                .fetch_one(&pool)
                .await?;
        assert_eq!(granted, expected, "iasion_app {privilege} on a new table");
    }
    Ok(())
}
