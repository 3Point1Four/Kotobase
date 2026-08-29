use domain::VocabularyEntry;
use sqlx::SqlitePool;

pub struct VocabularyRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> VocabularyRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        vocabulary: &VocabularyEntry,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO vocabulary (
                id
            )
            VALUES (?)
            "#,
        )
        .bind(vocabulary.id.as_uuid().to_string())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn exists(
        &self,
        vocabulary: &VocabularyEntry,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM vocabulary
            WHERE id = ?
            "#,
        )
        .bind(vocabulary.id.as_uuid().to_string())
        .fetch_one(self.pool)
        .await?;

        Ok(count > 0)
    }
}