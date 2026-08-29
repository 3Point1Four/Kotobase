use domain::GrammarPattern;
use sqlx::{Row, SqlitePool};

pub struct GrammarRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> GrammarRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        grammar: &GrammarPattern,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO grammar_patterns (
                id,
                name,
                formation,
                usage,
                source
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(grammar.id.as_uuid().to_string())
        .bind(&grammar.name)
        .bind(&grammar.formation)
        .bind(&grammar.usage)
        .bind(&grammar.source)
        .execute(&mut *transaction)
        .await?;

        for (position, meaning) in grammar.meanings.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO grammar_pattern_meanings (
                    grammar_id,
                    position,
                    meaning
                )
                VALUES (?, ?, ?)
                "#,
            )
            .bind(grammar.id.as_uuid().to_string())
            .bind(position as i64)
            .bind(meaning)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn get(
        &self,
        id: domain::EntityId,
    ) -> Result<Option<GrammarPattern>, sqlx::Error> {
        let id_string = id.as_uuid().to_string();

        let grammar_row = sqlx::query(
            r#"
            SELECT
                id,
                name,
                formation,
                usage,
                source
            FROM grammar_patterns
            WHERE id = ?
            "#,
        )
        .bind(&id_string)
        .fetch_optional(self.pool)
        .await?;

        let Some(row) = grammar_row else {
            return Ok(None);
        };

        let meaning_rows = sqlx::query(
            r#"
            SELECT meaning
            FROM grammar_pattern_meanings
            WHERE grammar_id = ?
            ORDER BY position
            "#,
        )
        .bind(&id_string)
        .fetch_all(self.pool)
        .await?;

        let meanings = meaning_rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("meaning"))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(GrammarPattern {
            id: parse_entity_id(&row.try_get::<String, _>("id")?)?,
            name: row.try_get("name")?,
            formation: row.try_get("formation")?,
            meanings,
            usage: row.try_get("usage")?,
            source: row.try_get("source")?,
        }))
    }

    pub async fn exists(
        &self,
        id: domain::EntityId,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM grammar_patterns
            WHERE id = ?
            "#,
        )
        .bind(id.as_uuid().to_string())
        .fetch_one(self.pool)
        .await?;

        Ok(count > 0)
    }

    pub async fn delete(
        &self,
        id: domain::EntityId,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM grammar_patterns
            WHERE id = ?
            "#,
        )
        .bind(id.as_uuid().to_string())
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

fn parse_entity_id(
    value: &str,
) -> Result<domain::EntityId, sqlx::Error> {
    uuid::Uuid::parse_str(value)
        .map(domain::EntityId::from_uuid)
        .map_err(|_| invalid_data("invalid grammar ID"))
}

fn invalid_data(message: &str) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message,
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::GrammarPattern;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .expect("failed to create test database");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");

        pool
    }

    fn sample_grammar() -> GrammarPattern {
        let mut grammar = GrammarPattern::new("〜ている");

        grammar.formation = "て-form + いる".to_string();
        grammar.meanings = vec![
            "to be doing".to_string(),
            "ongoing state".to_string(),
        ];
        grammar.usage = "Used to express an ongoing action or resulting state."
            .to_string();
        grammar.source = Some("Test Source".to_string());

        grammar
    }

    #[tokio::test]
    async fn insert_and_get_preserves_complete_grammar() {
        let pool = test_pool().await;
        let repository = GrammarRepository::new(&pool);

        let grammar = sample_grammar();

        repository
            .insert(&grammar)
            .await
            .expect("failed to insert grammar");

        let loaded = repository
            .get(grammar.id)
            .await
            .expect("failed to get grammar")
            .expect("grammar was not found");

        assert_eq!(loaded, grammar);
    }

    #[tokio::test]
    async fn get_returns_none_for_unknown_grammar() {
        let pool = test_pool().await;
        let repository = GrammarRepository::new(&pool);

        let id = domain::EntityId::new();

        let result = repository
            .get(id)
            .await
            .expect("failed to query grammar");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn exists_returns_true_for_inserted_grammar() {
        let pool = test_pool().await;
        let repository = GrammarRepository::new(&pool);

        let grammar = sample_grammar();

        repository
            .insert(&grammar)
            .await
            .expect("failed to insert grammar");

        assert!(
            repository
                .exists(grammar.id)
                .await
                .expect("failed to check existence")
        );
    }

    #[tokio::test]
    async fn exists_returns_false_for_unknown_grammar() {
        let pool = test_pool().await;
        let repository = GrammarRepository::new(&pool);

        let id = domain::EntityId::new();

        assert!(
            !repository
                .exists(id)
                .await
                .expect("failed to check existence")
        );
    }

    #[tokio::test]
    async fn delete_removes_grammar() {
        let pool = test_pool().await;
        let repository = GrammarRepository::new(&pool);

        let grammar = sample_grammar();

        repository
            .insert(&grammar)
            .await
            .expect("failed to insert grammar");

        assert!(
            repository
                .delete(grammar.id)
                .await
                .expect("failed to delete grammar")
        );

        assert!(
            repository
                .get(grammar.id)
                .await
                .expect("failed to get grammar")
                .is_none()
        );
    }

    #[tokio::test]
    async fn delete_returns_false_for_unknown_grammar() {
        let pool = test_pool().await;
        let repository = GrammarRepository::new(&pool);

        let id = domain::EntityId::new();

        assert!(
            !repository
                .delete(id)
                .await
                .expect("failed to delete grammar")
        );
    }

    #[tokio::test]
    async fn meanings_preserve_order() {
        let pool = test_pool().await;
        let repository = GrammarRepository::new(&pool);

        let mut grammar = GrammarPattern::new("test");
        grammar.meanings = vec![
            "first meaning".to_string(),
            "second meaning".to_string(),
            "third meaning".to_string(),
        ];

        repository
            .insert(&grammar)
            .await
            .expect("failed to insert grammar");

        let loaded = repository
            .get(grammar.id)
            .await
            .expect("failed to get grammar")
            .expect("grammar was not found");

        assert_eq!(
            loaded.meanings,
            vec![
                "first meaning".to_string(),
                "second meaning".to_string(),
                "third meaning".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn optional_source_can_be_none() {
        let pool = test_pool().await;
        let repository = GrammarRepository::new(&pool);

        let mut grammar = GrammarPattern::new("test");
        grammar.source = None;

        repository
            .insert(&grammar)
            .await
            .expect("failed to insert grammar");

        let loaded = repository
            .get(grammar.id)
            .await
            .expect("failed to get grammar")
            .expect("grammar was not found");

        assert_eq!(loaded.source, None);
    }

    #[tokio::test]
    async fn deleting_grammar_cascades_to_meanings() {
        let pool = test_pool().await;
        let repository = GrammarRepository::new(&pool);

        let mut grammar = GrammarPattern::new("test");
        grammar.meanings = vec![
            "first".to_string(),
            "second".to_string(),
        ];

        repository
            .insert(&grammar)
            .await
            .expect("failed to insert grammar");

        repository
            .delete(grammar.id)
            .await
            .expect("failed to delete grammar");

        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM grammar_pattern_meanings
            WHERE grammar_id = ?
            "#,
        )
        .bind(grammar.id.as_uuid().to_string())
        .fetch_one(&pool)
        .await
        .expect("failed to count meanings");

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn duplicate_grammar_id_fails_without_corrupting_existing_data() {
        let pool = test_pool().await;
        let repository = GrammarRepository::new(&pool);

        let grammar = sample_grammar();

        repository
            .insert(&grammar)
            .await
            .expect("failed to insert first grammar");

        let duplicate = GrammarPattern {
            id: grammar.id,
            name: "different".to_string(),
            formation: "different".to_string(),
            meanings: vec!["different".to_string()],
            usage: "different".to_string(),
            source: None,
        };

        assert!(
            repository
                .insert(&duplicate)
                .await
                .is_err()
        );

        let loaded = repository
            .get(grammar.id)
            .await
            .expect("failed to get grammar")
            .expect("original grammar disappeared");

        assert_eq!(loaded, grammar);
    }
}