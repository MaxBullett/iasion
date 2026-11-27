//! Which migrations are pending decides whether a backup is taken before `iasion
//! migrate` (ADR-0040, decisions 9 and 12), and whether the app may start.

use sqlx::PgPool;

fn embedded() -> usize {
    iasion_migrations::MIGRATOR.iter().count()
}

#[sqlx::test(migrations = false)]
async fn every_migration_is_pending_before_the_first_migrate(pool: PgPool) -> sqlx::Result<()> {
    assert_eq!(iasion_migrations::pending(&pool).await?, embedded());
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn none_is_pending_after_migrate(pool: PgPool) -> sqlx::Result<()> {
    iasion_migrations::release().run(&pool).await?;
    assert_eq!(iasion_migrations::pending(&pool).await?, 0);
    Ok(())
}

/// After a rollback: the newer release's migration is not this binary's to apply.
#[sqlx::test(migrations = false)]
async fn a_newer_releases_migration_is_not_pending(pool: PgPool) -> sqlx::Result<()> {
    iasion_migrations::release().run(&pool).await?;
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) \
         VALUES (99991231235959, 'household from a newer release', true, '\\x00', 0)",
    )
    .execute(&pool)
    .await?;
    assert_eq!(iasion_migrations::pending(&pool).await?, 0);
    Ok(())
}

#[sqlx::test(migrations = false)]
async fn a_failed_migration_is_still_pending(pool: PgPool) -> sqlx::Result<()> {
    iasion_migrations::release().run(&pool).await?;
    sqlx::query(
        "UPDATE _sqlx_migrations SET success = false \
         WHERE version = (SELECT max(version) FROM _sqlx_migrations)",
    )
    .execute(&pool)
    .await?;
    assert_eq!(iasion_migrations::pending(&pool).await?, 1);
    Ok(())
}
