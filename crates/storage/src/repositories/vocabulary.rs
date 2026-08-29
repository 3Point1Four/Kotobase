use domain::{PartOfSpeech, VocabularyEntry};
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
        let row = sqlx::query(
            r#"
            SELECT
                id,
                source
            FROM vocabulary
            WHERE id = ?
            "#,
        )
        .bind(id.as_uuid().to_string())
        .fetch_optional(self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id_string: String = sqlx::Row::try_get(&row, "id")?;
        let source: Option<String> =
            sqlx::Row::try_get(&row, "source")?;

        let written_forms = sqlx::query_scalar::<_, String>(
            r#"
            SELECT form
            FROM vocabulary_written_forms
            WHERE vocabulary_id = ?
            ORDER BY position
            "#,
        )
        .bind(id_string.clone())
        .fetch_all(self.pool)
        .await?;

        let readings = sqlx::query_scalar::<_, String>(
            r#"
            SELECT reading
            FROM vocabulary_readings
            WHERE vocabulary_id = ?
            ORDER BY position
            "#,
        )
        .bind(id_string.clone())
        .fetch_all(self.pool)
        .await?;

        let meanings = sqlx::query_scalar::<_, String>(
            r#"
            SELECT meaning
            FROM vocabulary_meanings
            WHERE vocabulary_id = ?
            ORDER BY position
            "#,
        )
        .bind(id_string.clone())
        .fetch_all(self.pool)
        .await?;

        let pos_strings = sqlx::query_scalar::<_, String>(
            r#"
            SELECT part_of_speech
            FROM vocabulary_parts_of_speech
            WHERE vocabulary_id = ?
            ORDER BY position
            "#,
        )
        .bind(id_string)
        .fetch_all(self.pool)
        .await?;

        let parts_of_speech = pos_strings
            .into_iter()
            .map(|value| part_of_speech_from_string(&value))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(VocabularyEntry {
            id,
            written_forms,
            readings,
            meanings,
            parts_of_speech,
            source,
        }))
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
        _ => Err(sqlx::Error::Decode(Box::new(
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unknown part of speech",
            ),
        ))),
    }
}