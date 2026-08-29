use domain::{EntityId, EntityType, Reference, ReferenceLocation};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

pub struct ReferenceRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> ReferenceRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        reference: &Reference,
    ) -> Result<(), sqlx::Error> {
        let (location_type, start_offset, end_offset, page, block_id) =
            match &reference.location {
                ReferenceLocation::CharacterRange { start, end } => (
                    "character_range",
                    Some(*start as i64),
                    Some(*end as i64),
                    None::<i64>,
                    None::<String>,
                ),

                ReferenceLocation::TokenRange { start, end } => (
                    "token_range",
                    Some(*start as i64),
                    Some(*end as i64),
                    None::<i64>,
                    None::<String>,
                ),

                ReferenceLocation::Page { page } => (
                    "page",
                    None::<i64>,
                    None::<i64>,
                    Some(*page as i64),
                    None::<String>,
                ),

                ReferenceLocation::Block { block_id } => (
                    "block",
                    None::<i64>,
                    None::<i64>,
                    None::<i64>,
                    Some(block_id.as_uuid().to_string()),
                ),

                ReferenceLocation::FileLocation {
                    page,
                    start,
                    end,
                } => (
                    "file_location",
                    start.map(|value| value as i64),
                    end.map(|value| value as i64),
                    page.map(|value| value as i64),
                    None::<String>,
                ),
            };

        sqlx::query(
            r#"
            INSERT INTO entity_references (
                id,
                source_id,
                source_type,
                target_id,
                target_type,
                location_type,
                start_offset,
                end_offset,
                page,
                block_id
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(reference.id.as_uuid().to_string())
        .bind(reference.source.as_uuid().to_string())
        .bind(entity_type_to_string(reference.source_type))
        .bind(reference.target.as_uuid().to_string())
        .bind(entity_type_to_string(reference.target_type))
        .bind(location_type)
        .bind(start_offset)
        .bind(end_offset)
        .bind(page)
        .bind(block_id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn get(
        &self,
        id: EntityId,
    ) -> Result<Option<Reference>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                source_id,
                source_type,
                target_id,
                target_type,
                location_type,
                start_offset,
                end_offset,
                page,
                block_id
            FROM entity_references
            WHERE id = ?
            "#,
        )
        .bind(id.as_uuid().to_string())
        .fetch_optional(self.pool)
        .await?;

        row.map(reference_from_row).transpose()
    }

    pub async fn delete(
        &self,
        id: EntityId,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM entity_references WHERE id = ?",
        )
        .bind(id.as_uuid().to_string())
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn by_source(
        &self,
        source: EntityId,
    ) -> Result<Vec<Reference>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                source_id,
                source_type,
                target_id,
                target_type,
                location_type,
                start_offset,
                end_offset,
                page,
                block_id
            FROM entity_references
            WHERE source_id = ?
            ORDER BY id
            "#,
        )
        .bind(source.as_uuid().to_string())
        .fetch_all(self.pool)
        .await?;

        rows.into_iter()
            .map(reference_from_row)
            .collect()
    }

    pub async fn by_target(
        &self,
        target: EntityId,
    ) -> Result<Vec<Reference>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                source_id,
                source_type,
                target_id,
                target_type,
                location_type,
                start_offset,
                end_offset,
                page,
                block_id
            FROM entity_references
            WHERE target_id = ?
            ORDER BY id
            "#,
        )
        .bind(target.as_uuid().to_string())
        .fetch_all(self.pool)
        .await?;

        rows.into_iter()
            .map(reference_from_row)
            .collect()
    }
}

fn reference_from_row(
    row: sqlx::sqlite::SqliteRow,
) -> Result<Reference, sqlx::Error> {
    let id = parse_entity_id(row.try_get::<String, _>("id")?)?;

    let source =
        parse_entity_id(row.try_get::<String, _>("source_id")?)?;

    let target =
        parse_entity_id(row.try_get::<String, _>("target_id")?)?;

    let source_type = entity_type_from_string(
        row.try_get::<String, _>("source_type")?,
    )?;

    let target_type = entity_type_from_string(
        row.try_get::<String, _>("target_type")?,
    )?;

    let location_type: String =
        row.try_get("location_type")?;

    let start_offset: Option<i64> =
        row.try_get("start_offset")?;

    let end_offset: Option<i64> =
        row.try_get("end_offset")?;

    let page: Option<i64> =
        row.try_get("page")?;

    let block_id: Option<String> =
        row.try_get("block_id")?;

    let location = match location_type.as_str() {
        "character_range" => {
            ReferenceLocation::CharacterRange {
                start: start_offset
                    .ok_or_else(|| {
                        invalid_data("missing start offset")
                    })?
                    as usize,

                end: end_offset
                    .ok_or_else(|| {
                        invalid_data("missing end offset")
                    })?
                    as usize,
            }
        }

        "token_range" => {
            ReferenceLocation::TokenRange {
                start: start_offset
                    .ok_or_else(|| {
                        invalid_data("missing start offset")
                    })?
                    as usize,

                end: end_offset
                    .ok_or_else(|| {
                        invalid_data("missing end offset")
                    })?
                    as usize,
            }
        }

        "page" => {
            ReferenceLocation::Page {
                page: page
                    .ok_or_else(|| {
                        invalid_data("missing page")
                    })?
                    as u32,
            }
        }

        "block" => {
            let block_id = block_id.ok_or_else(|| {
                invalid_data("missing block ID")
            })?;

            ReferenceLocation::Block {
                block_id: parse_entity_id(block_id)?,
            }
        }

        "file_location" => {
            ReferenceLocation::FileLocation {
                page: page.map(|value| value as u32),
                start: start_offset.map(|value| value as usize),
                end: end_offset.map(|value| value as usize),
            }
        }

        _ => {
            return Err(invalid_data(
                "unknown reference location type",
            ));
        }
    };

    Ok(Reference {
        id,
        source,
        source_type,
        target,
        target_type,
        location,
    })
}

fn parse_entity_id(
    value: String,
) -> Result<EntityId, sqlx::Error> {
    Uuid::parse_str(&value)
        .map(EntityId::from_uuid)
        .map_err(|_| invalid_data("invalid entity ID"))
}

fn invalid_data(message: &str) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message,
        ),
    ))
}

fn entity_type_to_string(
    entity_type: EntityType,
) -> &'static str {
    match entity_type {
        EntityType::Vocabulary => "vocabulary",
        EntityType::Kanji => "kanji",
        EntityType::Grammar => "grammar",
        EntityType::Sentence => "sentence",
        EntityType::Note => "note",
        EntityType::Page => "page",
        EntityType::File => "file",
        EntityType::Tag => "tag",
        EntityType::Meaning => "meaning",
        EntityType::Proverb => "proverb",
        EntityType::StudySession => "study_session",
    }
}

fn entity_type_from_string(
    value: String,
) -> Result<EntityType, sqlx::Error> {
    match value.as_str() {
        "vocabulary" => Ok(EntityType::Vocabulary),
        "kanji" => Ok(EntityType::Kanji),
        "grammar" => Ok(EntityType::Grammar),
        "sentence" => Ok(EntityType::Sentence),
        "note" => Ok(EntityType::Note),
        "page" => Ok(EntityType::Page),
        "file" => Ok(EntityType::File),
        "tag" => Ok(EntityType::Tag),
        "meaning" => Ok(EntityType::Meaning),
        "proverb" => Ok(EntityType::Proverb),
        "study_session" => Ok(EntityType::StudySession),
        _ => Err(invalid_data("unknown entity type")),
    }
}