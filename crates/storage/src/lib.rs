use sqlx::sqlite::SqlitePool;

/// Creates a SQLite connection pool.
pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePool::connect(database_url).await
}

/// Runs all pending database migrations.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migrations_create_database_schema() {
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("failed to create SQLite pool");

        run_migrations(&pool)
            .await
            .expect("failed to run migrations");

        let relationship_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM relationships")
                .fetch_one(&pool)
                .await
                .expect("failed to query relationships");

        let reference_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM entity_references")
                .fetch_one(&pool)
                .await
                .expect("failed to query references");

        assert_eq!(relationship_count, 0);
        assert_eq!(reference_count, 0);
    }

    #[tokio::test]
    async fn migrations_are_idempotent() {
        let pool = create_pool("sqlite::memory:")
            .await
            .expect("failed to create SQLite pool");

        run_migrations(&pool)
            .await
            .expect("first migration failed");

        run_migrations(&pool)
            .await
            .expect("second migration failed");
    }
}