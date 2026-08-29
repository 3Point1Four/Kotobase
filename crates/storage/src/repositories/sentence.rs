use domain::{GrammarMatch, Sentence, Token};
use sqlx::{Row, SqlitePool};

pub struct SentenceRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SentenceRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        sentence: &Sentence,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO sentences (
                id,
                text,
                translation,
                source
            )
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(sentence.id.as_uuid().to_string())
        .bind(&sentence.text)
        .bind(&sentence.translation)
        .bind(&sentence.source)
        .execute(&mut *transaction)
        .await?;

        for (position, token) in sentence.tokens.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO sentence_tokens (
                    id,
                    sentence_id,
                    position,
                    surface,
                    start_offset,
                    end_offset,
                    reading,
                    lemma,
                    part_of_speech,
                    vocabulary_id
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(token.id.as_uuid().to_string())
            .bind(sentence.id.as_uuid().to_string())
            .bind(position as i64)
            .bind(&token.surface)
            .bind(token.start as i64)
            .bind(token.end as i64)
            .bind(&token.reading)
            .bind(&token.lemma)
            .bind(&token.part_of_speech)
            .bind(token.vocabulary_id.map(|id| id.as_uuid().to_string()))
            .execute(&mut *transaction)
            .await?;
        }

        for grammar_match in &sentence.grammar_matches {
            sqlx::query(
                r#"
                INSERT INTO sentence_grammar_matches (
                    id,
                    sentence_id,
                    grammar_id,
                    start_offset,
                    end_offset
                )
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(grammar_match.id.as_uuid().to_string())
            .bind(sentence.id.as_uuid().to_string())
            .bind(grammar_match.grammar_id.as_uuid().to_string())
            .bind(grammar_match.start as i64)
            .bind(grammar_match.end as i64)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn get(
        &self,
        id: domain::EntityId,
    ) -> Result<Option<Sentence>, sqlx::Error> {
        let id_string = id.as_uuid().to_string();

        let sentence_row = sqlx::query(
            r#"
            SELECT
                id,
                text,
                translation,
                source
            FROM sentences
            WHERE id = ?
            "#,
        )
        .bind(&id_string)
        .fetch_optional(self.pool)
        .await?;

        let Some(row) = sentence_row else {
            return Ok(None);
        };

        let token_rows = sqlx::query(
            r#"
            SELECT
                id,
                surface,
                start_offset,
                end_offset,
                reading,
                lemma,
                part_of_speech,
                vocabulary_id
            FROM sentence_tokens
            WHERE sentence_id = ?
            ORDER BY position
            "#,
        )
        .bind(&id_string)
        .fetch_all(self.pool)
        .await?;

        let mut tokens = Vec::with_capacity(token_rows.len());

        for row in token_rows {
            let vocabulary_id: Option<String> =
                row.try_get("vocabulary_id")?;

            tokens.push(Token {
                id: parse_entity_id(&row.try_get::<String, _>("id")?)?,
                surface: row.try_get("surface")?,
                start: row.try_get::<i64, _>("start_offset")? as usize,
                end: row.try_get::<i64, _>("end_offset")? as usize,
                reading: row.try_get("reading")?,
                lemma: row.try_get("lemma")?,
                part_of_speech: row.try_get("part_of_speech")?,
                vocabulary_id: vocabulary_id
                    .as_deref()
                    .map(parse_entity_id)
                    .transpose()?,
            });
        }

        let grammar_rows = sqlx::query(
            r#"
            SELECT
                id,
                grammar_id,
                start_offset,
                end_offset
            FROM sentence_grammar_matches
            WHERE sentence_id = ?
            ORDER BY start_offset, end_offset, id
            "#,
        )
        .bind(&id_string)
        .fetch_all(self.pool)
        .await?;

        let mut grammar_matches = Vec::with_capacity(grammar_rows.len());

        for row in grammar_rows {
            grammar_matches.push(GrammarMatch {
                id: parse_entity_id(&row.try_get::<String, _>("id")?)?,
                grammar_id: parse_entity_id(
                    &row.try_get::<String, _>("grammar_id")?,
                )?,
                start: row.try_get::<i64, _>("start_offset")? as usize,
                end: row.try_get::<i64, _>("end_offset")? as usize,
            });
        }

        Ok(Some(Sentence {
            id: parse_entity_id(&row.try_get::<String, _>("id")?)?,
            text: row.try_get("text")?,
            translation: row.try_get("translation")?,
            source: row.try_get("source")?,
            tokens,
            grammar_matches,
        }))
    }

    pub async fn exists(
        &self,
        id: domain::EntityId,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM sentences
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
            DELETE FROM sentences
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
        .map_err(|_| invalid_data("invalid sentence entity ID"))
}

fn invalid_data(message: &str) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message,
        ),
    ))
}