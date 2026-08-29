use domain::{PartOfSpeech, VocabularyEntry};
use sqlx::{Row, SqlitePool};

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
        let mut transaction = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO vocabulary (
                id,
                source
            )
            VALUES (?, ?)
            "#,
        )
        .bind(vocabulary.id.as_uuid().to_string())
        .bind(&vocabulary.source)
        .execute(&mut *transaction)
        .await?;

        for (position, form) in vocabulary.written_forms.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO vocabulary_written_forms (
                    vocabulary_id,
                    position,
                    form
                )
                VALUES (?, ?, ?)
                "#,
            )
            .bind(vocabulary.id.as_uuid().to_string())
            .bind(position as i64)
            .bind(form)
            .execute(&mut *transaction)
            .await?;
        }

        for (position, reading) in vocabulary.readings.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO vocabulary_readings (
                    vocabulary_id,
                    position,
                    reading
                )
                VALUES (?, ?, ?)
                "#,
            )
            .bind(vocabulary.id.as_uuid().to_string())
            .bind(position as i64)
            .bind(reading)
            .execute(&mut *transaction)
            .await?;
        }

        for (position, meaning) in vocabulary.meanings.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO vocabulary_meanings (
                    vocabulary_id,
                    position,
                    meaning
                )
                VALUES (?, ?, ?)
                "#,
            )
            .bind(vocabulary.id.as_uuid().to_string())
            .bind(position as i64)
            .bind(meaning)
            .execute(&mut *transaction)
            .await?;
        }

        for (position, part_of_speech) in
            vocabulary.parts_of_speech.iter().enumerate()
        {
            sqlx::query(
                r#"
                INSERT INTO vocabulary_parts_of_speech (
                    vocabulary_id,
                    position,
                    part_of_speech
                )
                VALUES (?, ?, ?)
                "#,
            )
            .bind(vocabulary.id.as_uuid().to_string())
            .bind(position as i64)
            .bind(part_of_speech_to_string(*part_of_speech))
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn get(
        &self,
        id: domain::EntityId,
    ) -> Result<Option<VocabularyEntry>, sqlx::Error> {
        let id_string = id.as_uuid().to_string();

        let vocabulary_row = sqlx::query(
            r#"
            SELECT
                id,
                source
            FROM vocabulary
            WHERE id = ?
            "#,
        )
        .bind(&id_string)
        .fetch_optional(self.pool)
        .await?;

        let Some(row) = vocabulary_row else {
            return Ok(None);
        };

        let source: Option<String> = row.try_get("source")?;

        let written_form_rows = sqlx::query(
            r#"
            SELECT form
            FROM vocabulary_written_forms
            WHERE vocabulary_id = ?
            ORDER BY position
            "#,
        )
        .bind(&id_string)
        .fetch_all(self.pool)
        .await?;

        let written_forms = written_form_rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("form"))
            .collect::<Result<Vec<_>, _>>()?;

        let reading_rows = sqlx::query(
            r#"
            SELECT reading
            FROM vocabulary_readings
            WHERE vocabulary_id = ?
            ORDER BY position
            "#,
        )
        .bind(&id_string)
        .fetch_all(self.pool)
        .await?;

        let readings = reading_rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("reading"))
            .collect::<Result<Vec<_>, _>>()?;

        let meaning_rows = sqlx::query(
            r#"
            SELECT meaning
            FROM vocabulary_meanings
            WHERE vocabulary_id = ?
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

        let part_of_speech_rows = sqlx::query(
            r#"
            SELECT part_of_speech
            FROM vocabulary_parts_of_speech
            WHERE vocabulary_id = ?
            ORDER BY position
            "#,
        )
        .bind(&id_string)
        .fetch_all(self.pool)
        .await?;

        let parts_of_speech = part_of_speech_rows
            .into_iter()
            .map(|row| {
                let value = row.try_get::<String, _>("part_of_speech")?;
                part_of_speech_from_string(&value)
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;

        Ok(Some(VocabularyEntry {
            id: row
                .try_get::<String, _>("id")
                .and_then(|value| parse_entity_id(&value))?,
            written_forms,
            readings,
            meanings,
            parts_of_speech,
            source,
        }))
    }

    pub async fn exists(
        &self,
        id: domain::EntityId,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM vocabulary
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
            DELETE FROM vocabulary
            WHERE id = ?
            "#,
        )
        .bind(id.as_uuid().to_string())
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

fn part_of_speech_to_string(
    part_of_speech: PartOfSpeech,
) -> &'static str {
    match part_of_speech {
        PartOfSpeech::Noun => "noun",
        PartOfSpeech::Verb => "verb",
        PartOfSpeech::IAdjective => "i_adjective",
        PartOfSpeech::NaAdjective => "na_adjective",
        PartOfSpeech::Adverb => "adverb",
        PartOfSpeech::Particle => "particle",
        PartOfSpeech::Auxiliary => "auxiliary",
        PartOfSpeech::Conjunction => "conjunction",
        PartOfSpeech::Interjection => "interjection",
        PartOfSpeech::Pronoun => "pronoun",
        PartOfSpeech::Determiner => "determiner",
        PartOfSpeech::Counter => "counter",
        PartOfSpeech::Prefix => "prefix",
        PartOfSpeech::Suffix => "suffix",
        PartOfSpeech::Other => "other",
    }
}

fn part_of_speech_from_string(
    value: &str,
) -> Result<PartOfSpeech, sqlx::Error> {
    match value {
        "noun" => Ok(PartOfSpeech::Noun),
        "verb" => Ok(PartOfSpeech::Verb),
        "i_adjective" => Ok(PartOfSpeech::IAdjective),
        "na_adjective" => Ok(PartOfSpeech::NaAdjective),
        "adverb" => Ok(PartOfSpeech::Adverb),
        "particle" => Ok(PartOfSpeech::Particle),
        "auxiliary" => Ok(PartOfSpeech::Auxiliary),
        "conjunction" => Ok(PartOfSpeech::Conjunction),
        "interjection" => Ok(PartOfSpeech::Interjection),
        "pronoun" => Ok(PartOfSpeech::Pronoun),
        "determiner" => Ok(PartOfSpeech::Determiner),
        "counter" => Ok(PartOfSpeech::Counter),
        "prefix" => Ok(PartOfSpeech::Prefix),
        "suffix" => Ok(PartOfSpeech::Suffix),
        "other" => Ok(PartOfSpeech::Other),
        _ => Err(invalid_data("unknown part of speech")),
    }
}

fn parse_entity_id(
    value: &str,
) -> Result<domain::EntityId, sqlx::Error> {
    uuid::Uuid::parse_str(value)
        .map(domain::EntityId::from_uuid)
        .map_err(|_| invalid_data("invalid vocabulary ID"))
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
    use domain::{EntityId, PartOfSpeech, VocabularyEntry};
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .expect("failed to create in-memory SQLite database");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("failed to run migrations");

        pool
    }

    fn sample_vocabulary() -> VocabularyEntry {
        VocabularyEntry {
            id: EntityId::new(),
            written_forms: vec![
                "食べる".to_string(),
                "喰べる".to_string(),
            ],
            readings: vec![
                "たべる".to_string(),
                "タベル".to_string(),
            ],
            meanings: vec![
                "to eat".to_string(),
                "to consume".to_string(),
            ],
            parts_of_speech: vec![
                PartOfSpeech::Verb,
                PartOfSpeech::Other,
            ],
            source: Some("test source".to_string()),
        }
    }

    #[tokio::test]
    async fn insert_and_get_preserves_complete_vocabulary() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let vocabulary = sample_vocabulary();

        repository
            .insert(&vocabulary)
            .await
            .expect("failed to insert vocabulary");

        let retrieved = repository
            .get(vocabulary.id)
            .await
            .expect("failed to retrieve vocabulary")
            .expect("vocabulary was not found");

        assert_eq!(retrieved, vocabulary);
    }

    #[tokio::test]
    async fn written_forms_preserve_order() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let mut vocabulary = VocabularyEntry::new();

        vocabulary.written_forms = vec![
            "一".to_string(),
            "壱".to_string(),
            "いち".to_string(),
        ];

        repository
            .insert(&vocabulary)
            .await
            .expect("failed to insert vocabulary");

        let retrieved = repository
            .get(vocabulary.id)
            .await
            .expect("failed to retrieve vocabulary")
            .expect("vocabulary was not found");

        assert_eq!(
            retrieved.written_forms,
            vec![
                "一".to_string(),
                "壱".to_string(),
                "いち".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn readings_preserve_order() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let mut vocabulary = VocabularyEntry::new();

        vocabulary.readings = vec![
            "ひと".to_string(),
            "いち".to_string(),
            "いつ".to_string(),
        ];

        repository
            .insert(&vocabulary)
            .await
            .expect("failed to insert vocabulary");

        let retrieved = repository
            .get(vocabulary.id)
            .await
            .expect("failed to retrieve vocabulary")
            .expect("vocabulary was not found");

        assert_eq!(
            retrieved.readings,
            vec![
                "ひと".to_string(),
                "いち".to_string(),
                "いつ".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn meanings_preserve_order() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let mut vocabulary = VocabularyEntry::new();

        vocabulary.meanings = vec![
            "one".to_string(),
            "single".to_string(),
            "one thing".to_string(),
        ];

        repository
            .insert(&vocabulary)
            .await
            .expect("failed to insert vocabulary");

        let retrieved = repository
            .get(vocabulary.id)
            .await
            .expect("failed to retrieve vocabulary")
            .expect("vocabulary was not found");

        assert_eq!(
            retrieved.meanings,
            vec![
                "one".to_string(),
                "single".to_string(),
                "one thing".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn parts_of_speech_preserve_order() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let mut vocabulary = VocabularyEntry::new();

        vocabulary.parts_of_speech = vec![
            PartOfSpeech::Noun,
            PartOfSpeech::Verb,
            PartOfSpeech::IAdjective,
            PartOfSpeech::NaAdjective,
            PartOfSpeech::Adverb,
            PartOfSpeech::Particle,
            PartOfSpeech::Auxiliary,
            PartOfSpeech::Conjunction,
            PartOfSpeech::Interjection,
            PartOfSpeech::Pronoun,
            PartOfSpeech::Determiner,
            PartOfSpeech::Counter,
            PartOfSpeech::Prefix,
            PartOfSpeech::Suffix,
            PartOfSpeech::Other,
        ];

        repository
            .insert(&vocabulary)
            .await
            .expect("failed to insert vocabulary");

        let retrieved = repository
            .get(vocabulary.id)
            .await
            .expect("failed to retrieve vocabulary")
            .expect("vocabulary was not found");

        assert_eq!(
            retrieved.parts_of_speech,
            vocabulary.parts_of_speech
        );
    }

    #[tokio::test]
    async fn exists_returns_true_for_inserted_vocabulary() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let vocabulary = sample_vocabulary();

        repository
            .insert(&vocabulary)
            .await
            .expect("failed to insert vocabulary");

        assert!(
            repository
                .exists(vocabulary.id)
                .await
                .expect("failed to check vocabulary existence")
        );
    }

    #[tokio::test]
    async fn exists_returns_false_for_unknown_vocabulary() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let unknown_id = EntityId::new();

        assert!(
            !repository
                .exists(unknown_id)
                .await
                .expect("failed to check vocabulary existence")
        );
    }

    #[tokio::test]
    async fn get_returns_none_for_unknown_vocabulary() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let unknown_id = EntityId::new();

        let result = repository
            .get(unknown_id)
            .await
            .expect("failed to retrieve vocabulary");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn delete_removes_vocabulary() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let vocabulary = sample_vocabulary();

        repository
            .insert(&vocabulary)
            .await
            .expect("failed to insert vocabulary");

        assert!(
            repository
                .delete(vocabulary.id)
                .await
                .expect("failed to delete vocabulary")
        );

        assert!(
            !repository
                .exists(vocabulary.id)
                .await
                .expect("failed to check vocabulary existence")
        );

        assert!(
            repository
                .get(vocabulary.id)
                .await
                .expect("failed to retrieve vocabulary")
                .is_none()
        );
    }

    #[tokio::test]
    async fn delete_returns_false_for_unknown_vocabulary() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let unknown_id = EntityId::new();

        assert!(
            !repository
                .delete(unknown_id)
                .await
                .expect("failed to delete vocabulary")
        );
    }

    #[tokio::test]
    async fn deleting_vocabulary_cascades_to_child_rows() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let vocabulary = sample_vocabulary();

        repository
            .insert(&vocabulary)
            .await
            .expect("failed to insert vocabulary");

        let id = vocabulary.id.as_uuid().to_string();

        repository
            .delete(vocabulary.id)
            .await
            .expect("failed to delete vocabulary");

        let written_forms: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM vocabulary_written_forms
            WHERE vocabulary_id = ?
            "#,
        )
        .bind(&id)
        .fetch_one(&pool)
        .await
        .expect("failed to count written forms");

        let readings: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM vocabulary_readings
            WHERE vocabulary_id = ?
            "#,
        )
        .bind(&id)
        .fetch_one(&pool)
        .await
        .expect("failed to count readings");

        let meanings: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM vocabulary_meanings
            WHERE vocabulary_id = ?
            "#,
        )
        .bind(&id)
        .fetch_one(&pool)
        .await
        .expect("failed to count meanings");

        let parts_of_speech: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM vocabulary_parts_of_speech
            WHERE vocabulary_id = ?
            "#,
        )
        .bind(&id)
        .fetch_one(&pool)
        .await
        .expect("failed to count parts of speech");

        assert_eq!(written_forms, 0);
        assert_eq!(readings, 0);
        assert_eq!(meanings, 0);
        assert_eq!(parts_of_speech, 0);
    }

    #[tokio::test]
    async fn vocabulary_without_optional_source_can_be_stored() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let vocabulary = VocabularyEntry::new();

        repository
            .insert(&vocabulary)
            .await
            .expect("failed to insert vocabulary");

        let retrieved = repository
            .get(vocabulary.id)
            .await
            .expect("failed to retrieve vocabulary")
            .expect("vocabulary was not found");

        assert_eq!(retrieved, vocabulary);
        assert!(retrieved.source.is_none());
    }

    #[tokio::test]
    async fn duplicate_vocabulary_id_fails_without_corrupting_existing_data() {
        let pool = test_pool().await;
        let repository = VocabularyRepository::new(&pool);

        let vocabulary = sample_vocabulary();

        repository
            .insert(&vocabulary)
            .await
            .expect("initial insert failed");

        let duplicate_result = repository.insert(&vocabulary).await;

        assert!(duplicate_result.is_err());

        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM vocabulary
            WHERE id = ?
            "#,
        )
        .bind(vocabulary.id.as_uuid().to_string())
        .fetch_one(&pool)
        .await
        .expect("failed to count vocabulary rows");

        assert_eq!(count, 1);

        let retrieved = repository
            .get(vocabulary.id)
            .await
            .expect("failed to retrieve existing vocabulary")
            .expect("existing vocabulary disappeared");

        assert_eq!(retrieved, vocabulary);
    }
}