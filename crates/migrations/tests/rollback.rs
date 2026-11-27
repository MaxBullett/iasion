//! After a rollback, the previous release runs on a schema migrated by the newer one
//! (ADR-0040, decision 10): its `migrate` must accept migrations it does not know.

use sqlx::PgPool;

/// What a newer release leaves behind: a migration this binary does not embed.
async fn record_a_newer_migration(pool: &PgPool) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) \
         VALUES (99991231235959, 'household from a newer release', true, '\\x00', 0)",
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn the_release_migrator_accepts_newer_migrations(pool: PgPool) -> sqlx::Result<()> {
    iasion_migrations::release().run(&pool).await?;
    record_a_newer_migration(&pool).await?;

    iasion_migrations::release().run(&pool).await?;
    Ok(())
}

/// Why the release migrator exists: sqlx's default refuses the database.
#[sqlx::test(migrations = false)]
async fn the_default_migrator_refuses_newer_migrations(pool: PgPool) -> sqlx::Result<()> {
    iasion_migrations::MIGRATOR.run(&pool).await?;
    record_a_newer_migration(&pool).await?;

    let error = iasion_migrations::MIGRATOR.run(&pool).await.unwrap_err();
    assert!(
        matches!(
            error,
            sqlx::migrate::MigrateError::VersionMissing(99991231235959)
        ),
        "{error}"
    );
    Ok(())
}
