pub mod repositories;

use sqlx::SqlitePool;

pub async fn initialize_database(
    pool: &SqlitePool,
) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    Ok(())
}