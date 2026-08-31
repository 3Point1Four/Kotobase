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

        pub async fn exists(
        &self,
        id: EntityId,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            SELECT 1
            FROM entity_references
            WHERE id = ?
            LIMIT 1
            "#,
        )
        .bind(id.as_uuid().to_string())
        .fetch_optional(self.pool)
        .await?;

        Ok(result.is_some())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initialize_database;
    use domain::{EntityType, Reference, ReferenceLocation};
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .expect("failed to create test database");

        initialize_database(&pool)
            .await
            .expect("failed to run migrations");

        pool
    }

    fn test_reference(
        location: ReferenceLocation,
    ) -> Reference {
        Reference::new(
            EntityId::new(),
            EntityType::Sentence,
            EntityId::new(),
            EntityType::Vocabulary,
            location,
        )
    }

    #[tokio::test]
    async fn insert_and_get_character_range() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        let reference = test_reference(
            ReferenceLocation::CharacterRange {
                start: 4,
                end: 9,
            },
        );

        repository
            .insert(&reference)
            .await
            .expect("failed to insert reference");

        let loaded = repository
            .get(reference.id)
            .await
            .expect("failed to get reference")
            .expect("reference was not found");

        assert_eq!(loaded, reference);
    }

    #[tokio::test]
    async fn insert_and_get_token_range() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        let reference = test_reference(
            ReferenceLocation::TokenRange {
                start: 2,
                end: 5,
            },
        );

        repository
            .insert(&reference)
            .await
            .expect("failed to insert reference");

        let loaded = repository
            .get(reference.id)
            .await
            .expect("failed to get reference")
            .expect("reference was not found");

        assert_eq!(loaded, reference);
    }

    #[tokio::test]
    async fn insert_and_get_page_location() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        let reference = test_reference(
            ReferenceLocation::Page {
                page: 17,
            },
        );

        repository
            .insert(&reference)
            .await
            .expect("failed to insert reference");

        let loaded = repository
            .get(reference.id)
            .await
            .expect("failed to get reference")
            .expect("reference was not found");

        assert_eq!(loaded, reference);
    }

    #[tokio::test]
    async fn insert_and_get_block_location() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        let block_id = EntityId::new();

        let reference = test_reference(
            ReferenceLocation::Block {
                block_id,
            },
        );

        repository
            .insert(&reference)
            .await
            .expect("failed to insert reference");

        let loaded = repository
            .get(reference.id)
            .await
            .expect("failed to get reference")
            .expect("reference was not found");

        assert_eq!(loaded, reference);
    }

    #[tokio::test]
    async fn insert_and_get_file_location() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        let reference = test_reference(
            ReferenceLocation::FileLocation {
                page: Some(3),
                start: Some(120),
                end: Some(145),
            },
        );

        repository
            .insert(&reference)
            .await
            .expect("failed to insert reference");

        let loaded = repository
            .get(reference.id)
            .await
            .expect("failed to get reference")
            .expect("reference was not found");

        assert_eq!(loaded, reference);
    }

    #[tokio::test]
    async fn file_location_preserves_optional_values() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        let reference = test_reference(
            ReferenceLocation::FileLocation {
                page: None,
                start: None,
                end: None,
            },
        );

        repository
            .insert(&reference)
            .await
            .expect("failed to insert reference");

        let loaded = repository
            .get(reference.id)
            .await
            .expect("failed to get reference")
            .expect("reference was not found");

        assert_eq!(loaded, reference);
    }

    #[tokio::test]
    async fn get_returns_none_for_unknown_reference() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        let result = repository
            .get(EntityId::new())
            .await
            .expect("failed to query reference");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn exists_returns_true_for_inserted_reference() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        let reference = test_reference(
            ReferenceLocation::Page { page: 1 },
        );

        repository
            .insert(&reference)
            .await
            .expect("failed to insert reference");

        assert!(
            repository
                .exists(reference.id)
                .await
                .expect("failed to check existence")
        );
    }

    #[tokio::test]
    async fn exists_returns_false_for_unknown_reference() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        assert!(
            !repository
                .exists(EntityId::new())
                .await
                .expect("failed to check existence")
        );
    }

    #[tokio::test]
    async fn delete_removes_reference() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        let reference = test_reference(
            ReferenceLocation::Page { page: 2 },
        );

        repository
            .insert(&reference)
            .await
            .expect("failed to insert reference");

        assert!(
            repository
                .delete(reference.id)
                .await
                .expect("failed to delete reference")
        );

        assert!(
            repository
                .get(reference.id)
                .await
                .expect("failed to get reference")
                .is_none()
        );
    }

    #[tokio::test]
    async fn delete_returns_false_for_unknown_reference() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        assert!(
            !repository
                .delete(EntityId::new())
                .await
                .expect("failed to delete reference")
        );
    }

    #[tokio::test]
    async fn by_source_returns_matching_references() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        let source = EntityId::new();

        let first = Reference::new(
            source,
            EntityType::Sentence,
            EntityId::new(),
            EntityType::Vocabulary,
            ReferenceLocation::CharacterRange {
                start: 0,
                end: 2,
            },
        );

        let second = Reference::new(
            source,
            EntityType::Sentence,
            EntityId::new(),
            EntityType::Kanji,
            ReferenceLocation::TokenRange {
                start: 3,
                end: 4,
            },
        );

        let unrelated = test_reference(
            ReferenceLocation::Page { page: 9 },
        );

        repository
            .insert(&first)
            .await
            .expect("failed to insert first reference");

        repository
            .insert(&second)
            .await
            .expect("failed to insert second reference");

        repository
            .insert(&unrelated)
            .await
            .expect("failed to insert unrelated reference");

        let references = repository
            .by_source(source)
            .await
            .expect("failed to query by source");

        assert_eq!(references.len(), 2);
        assert!(references.iter().any(|r| r.id == first.id));
        assert!(references.iter().any(|r| r.id == second.id));
        assert!(
            references
                .iter()
                .all(|r| r.id != unrelated.id)
        );
    }

    #[tokio::test]
    async fn by_target_returns_matching_references() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        let target = EntityId::new();

        let first = Reference::new(
            EntityId::new(),
            EntityType::Sentence,
            target,
            EntityType::Vocabulary,
            ReferenceLocation::Page { page: 1 },
        );

        let second = Reference::new(
            EntityId::new(),
            EntityType::Note,
            target,
            EntityType::Kanji,
            ReferenceLocation::Block {
                block_id: EntityId::new(),
            },
        );

        let unrelated = test_reference(
            ReferenceLocation::Page { page: 9 },
        );

        repository
            .insert(&first)
            .await
            .expect("failed to insert first reference");

        repository
            .insert(&second)
            .await
            .expect("failed to insert second reference");

        repository
            .insert(&unrelated)
            .await
            .expect("failed to insert unrelated reference");

        let references = repository
            .by_target(target)
            .await
            .expect("failed to query by target");

        assert_eq!(references.len(), 2);
        assert!(references.iter().any(|r| r.id == first.id));
        assert!(references.iter().any(|r| r.id == second.id));
        assert!(
            references
                .iter()
                .all(|r| r.id != unrelated.id)
        );
    }

    #[tokio::test]
    async fn duplicate_reference_id_fails() {
        let pool = test_pool().await;
        let repository = ReferenceRepository::new(&pool);

        let reference = test_reference(
            ReferenceLocation::Page { page: 1 },
        );

        repository
            .insert(&reference)
            .await
            .expect("failed to insert reference");

        let duplicate = Reference {
            id: reference.id,
            source: EntityId::new(),
            source_type: EntityType::Note,
            target: EntityId::new(),
            target_type: EntityType::Kanji,
            location: ReferenceLocation::Page { page: 2 },
        };

        assert!(
            repository.insert(&duplicate).await.is_err()
        );

        let loaded = repository
            .get(reference.id)
            .await
            .expect("failed to get original reference")
            .expect("original reference disappeared");

        assert_eq!(loaded, reference);
    }
}