use sqlx::sqlite::SqlitePool;

/// Creates a SQLite connection pool.
pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePool::connect(database_url).await
}