use domain::KanjiEntry;
use sqlx::{Row, SqlitePool};

pub struct KanjiRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> KanjiRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        kanji: &KanjiEntry,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO kanji (
                id,
                character,
                stroke_count,
                grade,
                jlpt_level
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(kanji.id.as_uuid().to_string())
        .bind(kanji.character.to_string())
        .bind(kanji.stroke_count.map(i64::from))
        .bind(kanji.grade.map(i64::from))
        .bind(kanji.jlpt_level.map(i64::from))
        .execute(&mut *transaction)
        .await?;

        for (position, reading) in kanji.on_readings.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO kanji_on_readings (
                    kanji_id,
                    position,
                    reading
                )
                VALUES (?, ?, ?)
                "#,
            )
            .bind(kanji.id.as_uuid().to_string())
            .bind(position as i64)
            .bind(reading)
            .execute(&mut *transaction)
            .await?;
        }

        for (position, reading) in kanji.kun_readings.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO kanji_kun_readings (
                    kanji_id,
                    position,
                    reading
                )
                VALUES (?, ?, ?)
                "#,
            )
            .bind(kanji.id.as_uuid().to_string())
            .bind(position as i64)
            .bind(reading)
            .execute(&mut *transaction)
            .await?;
        }

        for (position, meaning) in kanji.meanings.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO kanji_meanings (
                    kanji_id,
                    position,
                    meaning
                )
                VALUES (?, ?, ?)
                "#,
            )
            .bind(kanji.id.as_uuid().to_string())
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
    ) -> Result<Option<KanjiEntry>, sqlx::Error> {
        let id_string = id.as_uuid().to_string();

        let kanji_row = sqlx::query(
            r#"
            SELECT
                id,
                character,
                stroke_count,
                grade,
                jlpt_level
            FROM kanji
            WHERE id = ?
            "#,
        )
        .bind(&id_string)
        .fetch_optional(self.pool)
        .await?;

        let Some(row) = kanji_row else {
            return Ok(None);
        };

        let id_value: String = row.try_get("id")?;
        let character_value: String = row.try_get("character")?;

        let mut characters = character_value.chars();

        let character = characters
            .next()
            .ok_or_else(|| invalid_data("kanji character is empty"))?;

        if characters.next().is_some() {
            return Err(invalid_data(
                "kanji character contains more than one character",
            ));
        }

        let stroke_count: Option<i64> =
            row.try_get("stroke_count")?;

        let grade: Option<i64> =
            row.try_get("grade")?;

        let jlpt_level: Option<i64> =
            row.try_get("jlpt_level")?;

        let on_reading_rows = sqlx::query(
            r#"
            SELECT reading
            FROM kanji_on_readings
            WHERE kanji_id = ?
            ORDER BY position
            "#,
        )
        .bind(&id_string)
        .fetch_all(self.pool)
        .await?;

        let on_readings = on_reading_rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("reading"))
            .collect::<Result<Vec<_>, _>>()?;

        let kun_reading_rows = sqlx::query(
            r#"
            SELECT reading
            FROM kanji_kun_readings
            WHERE kanji_id = ?
            ORDER BY position
            "#,
        )
        .bind(&id_string)
        .fetch_all(self.pool)
        .await?;

        let kun_readings = kun_reading_rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("reading"))
            .collect::<Result<Vec<_>, _>>()?;

        let meaning_rows = sqlx::query(
            r#"
            SELECT meaning
            FROM kanji_meanings
            WHERE kanji_id = ?
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

        Ok(Some(KanjiEntry {
            id: parse_entity_id(&id_value)?,
            character,
            on_readings,
            kun_readings,
            meanings,
            stroke_count: stroke_count.map(|value| value as u16),
            grade: grade.map(|value| value as u8),
            jlpt_level: jlpt_level.map(|value| value as u8),
        }))
    }

    pub async fn exists(
        &self,
        id: domain::EntityId,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM kanji
            WHERE id = ?
            "#,
        )
        .bind(id.as_uuid().to_string())
        .fetch_one(self.pool)
        .await?;

        Ok(count > 0)
    }

    pub async fn get_by_character(
        &self,
        character: char,
    ) -> Result<Option<KanjiEntry>, sqlx::Error> {
        let id: Option<String> = sqlx::query_scalar(
            r#"
            SELECT id
            FROM kanji
            WHERE character = ?
            "#,
        )
        .bind(character.to_string())
        .fetch_optional(self.pool)
        .await?;

        let Some(id) = id else {
            return Ok(None);
        };

        let entity_id = parse_entity_id(&id)?;

        self.get(entity_id).await
    }

    pub async fn delete(
        &self,
        id: domain::EntityId,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM kanji
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
        .map_err(|_| invalid_data("invalid kanji ID"))
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
    use domain::EntityId;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:")
            .await
            .expect("failed to create test database");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");

        pool
    }

    fn complete_kanji() -> KanjiEntry {
        let mut kanji = KanjiEntry::new('学');

        kanji.on_readings = vec![
            "ガク".to_string(),
            "ガッ".to_string(),
        ];

        kanji.kun_readings = vec![
            "まな.ぶ".to_string(),
            "まな.び".to_string(),
        ];

        kanji.meanings = vec![
            "study".to_string(),
            "learning".to_string(),
        ];

        kanji.stroke_count = Some(8);
        kanji.grade = Some(1);
        kanji.jlpt_level = Some(5);

        kanji
    }

    #[tokio::test]
    async fn insert_and_get_preserves_complete_kanji() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        let kanji = complete_kanji();

        repository
            .insert(&kanji)
            .await
            .expect("failed to insert kanji");

        let loaded = repository
            .get(kanji.id)
            .await
            .expect("failed to get kanji")
            .expect("kanji was not found");

        assert_eq!(loaded, kanji);
    }

    #[tokio::test]
    async fn get_returns_none_for_unknown_kanji() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        let result = repository
            .get(EntityId::new())
            .await
            .expect("get should succeed");

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn exists_returns_true_for_inserted_kanji() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        let kanji = complete_kanji();

        repository
            .insert(&kanji)
            .await
            .expect("failed to insert kanji");

        assert!(
            repository
                .exists(kanji.id)
                .await
                .expect("exists should succeed")
        );
    }

    #[tokio::test]
    async fn exists_returns_false_for_unknown_kanji() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        assert!(
            !repository
                .exists(EntityId::new())
                .await
                .expect("exists should succeed")
        );
    }

    #[tokio::test]
    async fn get_by_character_returns_inserted_kanji() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        let kanji = complete_kanji();

        repository
            .insert(&kanji)
            .await
            .expect("failed to insert kanji");

        let loaded = repository
            .get_by_character('学')
            .await
            .expect("get_by_character should succeed")
            .expect("kanji was not found");

        assert_eq!(loaded, kanji);
    }

    #[tokio::test]
    async fn get_by_character_returns_none_for_unknown_character() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        let result = repository
            .get_by_character('龍')
            .await
            .expect("get_by_character should succeed");

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn on_readings_preserve_order() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        let mut kanji = KanjiEntry::new('日');

        kanji.on_readings = vec![
            "ニチ".to_string(),
            "ジツ".to_string(),
            "ニ".to_string(),
        ];

        repository
            .insert(&kanji)
            .await
            .expect("failed to insert kanji");

        let loaded = repository
            .get(kanji.id)
            .await
            .expect("failed to get kanji")
            .expect("kanji was not found");

        assert_eq!(
            loaded.on_readings,
            vec![
                "ニチ".to_string(),
                "ジツ".to_string(),
                "ニ".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn kun_readings_preserve_order() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        let mut kanji = KanjiEntry::new('日');

        kanji.kun_readings = vec![
            "ひ".to_string(),
            "か".to_string(),
            "-び".to_string(),
        ];

        repository
            .insert(&kanji)
            .await
            .expect("failed to insert kanji");

        let loaded = repository
            .get(kanji.id)
            .await
            .expect("failed to get kanji")
            .expect("kanji was not found");

        assert_eq!(
            loaded.kun_readings,
            vec![
                "ひ".to_string(),
                "か".to_string(),
                "-び".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn meanings_preserve_order() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        let mut kanji = KanjiEntry::new('学');

        kanji.meanings = vec![
            "study".to_string(),
            "learning".to_string(),
            "science".to_string(),
        ];

        repository
            .insert(&kanji)
            .await
            .expect("failed to insert kanji");

        let loaded = repository
            .get(kanji.id)
            .await
            .expect("failed to get kanji")
            .expect("kanji was not found");

        assert_eq!(
            loaded.meanings,
            vec![
                "study".to_string(),
                "learning".to_string(),
                "science".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn optional_metadata_can_be_none() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        let kanji = KanjiEntry::new('𠮷');

        repository
            .insert(&kanji)
            .await
            .expect("failed to insert kanji");

        let loaded = repository
            .get(kanji.id)
            .await
            .expect("failed to get kanji")
            .expect("kanji was not found");

        assert_eq!(loaded.stroke_count, None);
        assert_eq!(loaded.grade, None);
        assert_eq!(loaded.jlpt_level, None);
    }

    #[tokio::test]
    async fn delete_removes_kanji() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        let kanji = complete_kanji();

        repository
            .insert(&kanji)
            .await
            .expect("failed to insert kanji");

        assert!(
            repository
                .delete(kanji.id)
                .await
                .expect("delete should succeed")
        );

        assert!(
            !repository
                .exists(kanji.id)
                .await
                .expect("exists should succeed")
        );
    }

    #[tokio::test]
    async fn delete_returns_false_for_unknown_kanji() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        assert!(
            !repository
                .delete(EntityId::new())
                .await
                .expect("delete should succeed")
        );
    }

    #[tokio::test]
    async fn deleting_kanji_cascades_to_child_rows() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        let kanji = complete_kanji();

        repository
            .insert(&kanji)
            .await
            .expect("failed to insert kanji");

        repository
            .delete(kanji.id)
            .await
            .expect("delete should succeed");

        let on_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM kanji_on_readings
            WHERE kanji_id = ?
            "#,
        )
        .bind(kanji.id.as_uuid().to_string())
        .fetch_one(&pool)
        .await
        .expect("failed to count on readings");

        let kun_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM kanji_kun_readings
            WHERE kanji_id = ?
            "#,
        )
        .bind(kanji.id.as_uuid().to_string())
        .fetch_one(&pool)
        .await
        .expect("failed to count kun readings");

        let meaning_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM kanji_meanings
            WHERE kanji_id = ?
            "#,
        )
        .bind(kanji.id.as_uuid().to_string())
        .fetch_one(&pool)
        .await
        .expect("failed to count meanings");

        assert_eq!(on_count, 0);
        assert_eq!(kun_count, 0);
        assert_eq!(meaning_count, 0);
    }

    #[tokio::test]
    async fn duplicate_character_fails() {
        let pool = test_pool().await;
        let repository = KanjiRepository::new(&pool);

        let first = KanjiEntry::new('日');
        let second = KanjiEntry::new('日');

        repository
            .insert(&first)
            .await
            .expect("first insert should succeed");

        let result = repository.insert(&second).await;

        assert!(result.is_err());

        assert!(
            repository
                .exists(first.id)
                .await
                .expect("exists should succeed")
        );

        assert!(
            !repository
                .exists(second.id)
                .await
                .expect("exists should succeed")
        );
    }
}