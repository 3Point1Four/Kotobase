use domain::{
    EntityId,
    Relationship,
    RelationshipKind,
    RelationshipMetadata,
};
use sqlx::{Row, SqlitePool};

pub struct RelationshipRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> RelationshipRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        relationship: &Relationship,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO relationships (
                source_id,
                target_id,
                kind,
                label,
                context
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(relationship.source.as_uuid().to_string())
        .bind(relationship.target.as_uuid().to_string())
        .bind(relationship_kind_to_string(relationship.kind))
        .bind(&relationship.metadata.label)
        .bind(&relationship.metadata.context)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn get(
        &self,
        source: EntityId,
        target: EntityId,
        kind: RelationshipKind,
    ) -> Result<Option<Relationship>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                source_id,
                target_id,
                kind,
                label,
                context
            FROM relationships
            WHERE source_id = ?
              AND target_id = ?
              AND kind = ?
            "#,
        )
        .bind(source.as_uuid().to_string())
        .bind(target.as_uuid().to_string())
        .bind(relationship_kind_to_string(kind))
        .fetch_optional(self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(row_to_relationship(&row)?))
    }

    pub async fn by_source(
        &self,
        source: EntityId,
    ) -> Result<Vec<Relationship>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                source_id,
                target_id,
                kind,
                label,
                context
            FROM relationships
            WHERE source_id = ?
            ORDER BY target_id, kind
            "#,
        )
        .bind(source.as_uuid().to_string())
        .fetch_all(self.pool)
        .await?;

        rows.iter()
            .map(row_to_relationship)
            .collect()
    }

    pub async fn by_target(
        &self,
        target: EntityId,
    ) -> Result<Vec<Relationship>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                source_id,
                target_id,
                kind,
                label,
                context
            FROM relationships
            WHERE target_id = ?
            ORDER BY source_id, kind
            "#,
        )
        .bind(target.as_uuid().to_string())
        .fetch_all(self.pool)
        .await?;

        rows.iter()
            .map(row_to_relationship)
            .collect()
    }

    pub async fn delete(
        &self,
        source: EntityId,
        target: EntityId,
        kind: RelationshipKind,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM relationships
            WHERE source_id = ?
              AND target_id = ?
              AND kind = ?
            "#,
        )
        .bind(source.as_uuid().to_string())
        .bind(target.as_uuid().to_string())
        .bind(relationship_kind_to_string(kind))
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

fn relationship_kind_to_string(
    kind: RelationshipKind,
) -> &'static str {
    match kind {
        RelationshipKind::About => "about",
        RelationshipKind::Contains => "contains",
        RelationshipKind::References => "references",
        RelationshipKind::Uses => "uses",
        RelationshipKind::ExampleOf => "example_of",
        RelationshipKind::Explains => "explains",
        RelationshipKind::Translates => "translates",
        RelationshipKind::DerivedFrom => "derived_from",
        RelationshipKind::RelatedTo => "related_to",
    }
}

fn relationship_kind_from_string(
    value: &str,
) -> Result<RelationshipKind, sqlx::Error> {
    match value {
        "about" => Ok(RelationshipKind::About),
        "contains" => Ok(RelationshipKind::Contains),
        "references" => Ok(RelationshipKind::References),
        "uses" => Ok(RelationshipKind::Uses),
        "example_of" => Ok(RelationshipKind::ExampleOf),
        "explains" => Ok(RelationshipKind::Explains),
        "translates" => Ok(RelationshipKind::Translates),
        "derived_from" => Ok(RelationshipKind::DerivedFrom),
        "related_to" => Ok(RelationshipKind::RelatedTo),
        _ => Err(invalid_data("unknown relationship kind")),
    }
}

fn row_to_relationship(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<Relationship, sqlx::Error> {
    let source = parse_entity_id(
        row.try_get::<String, _>("source_id")?.as_str(),
    )?;

    let target = parse_entity_id(
        row.try_get::<String, _>("target_id")?.as_str(),
    )?;

    let kind = relationship_kind_from_string(
        row.try_get::<String, _>("kind")?.as_str(),
    )?;

    let metadata = RelationshipMetadata {
        label: row.try_get("label")?,
        context: row.try_get("context")?,
    };

    Ok(Relationship::with_metadata(
        source,
        target,
        kind,
        metadata,
    ))
}

fn parse_entity_id(
    value: &str,
) -> Result<EntityId, sqlx::Error> {
    uuid::Uuid::parse_str(value)
        .map(EntityId::from_uuid)
        .map_err(|_| invalid_data("invalid relationship entity ID"))
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
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .expect("failed to create test database");

        crate::initialize_database(&pool)
            .await
            .expect("failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn insert_and_get_preserves_relationship() {
        let pool = setup().await;
        let repository = RelationshipRepository::new(&pool);

        let relationship = Relationship::new(
            EntityId::new(),
            EntityId::new(),
            RelationshipKind::About,
        );

        repository
            .insert(&relationship)
            .await
            .expect("failed to insert relationship");

        let loaded = repository
            .get(
                relationship.source,
                relationship.target,
                relationship.kind,
            )
            .await
            .expect("failed to get relationship")
            .expect("relationship was not found");

        assert_eq!(loaded, relationship);
    }

    #[tokio::test]
    async fn metadata_is_preserved() {
        let pool = setup().await;
        let repository = RelationshipRepository::new(&pool);

        let relationship = Relationship::with_metadata(
            EntityId::new(),
            EntityId::new(),
            RelationshipKind::Explains,
            RelationshipMetadata {
                label: Some("grammar explanation".to_string()),
                context: Some("Used in this sentence".to_string()),
            },
        );

        repository
            .insert(&relationship)
            .await
            .expect("failed to insert relationship");

        let loaded = repository
            .get(
                relationship.source,
                relationship.target,
                relationship.kind,
            )
            .await
            .expect("failed to get relationship")
            .expect("relationship was not found");

        assert_eq!(loaded, relationship);
    }

    #[tokio::test]
    async fn get_returns_none_for_unknown_relationship() {
        let pool = setup().await;
        let repository = RelationshipRepository::new(&pool);

        let result = repository
            .get(
                EntityId::new(),
                EntityId::new(),
                RelationshipKind::RelatedTo,
            )
            .await
            .expect("failed to get relationship");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn by_source_returns_relationships() {
        let pool = setup().await;
        let repository = RelationshipRepository::new(&pool);

        let source = EntityId::new();

        let first = Relationship::new(
            source,
            EntityId::new(),
            RelationshipKind::About,
        );

        let second = Relationship::new(
            source,
            EntityId::new(),
            RelationshipKind::Contains,
        );

        repository
            .insert(&first)
            .await
            .expect("failed to insert first relationship");

        repository
            .insert(&second)
            .await
            .expect("failed to insert second relationship");

        let relationships = repository
            .by_source(source)
            .await
            .expect("failed to query relationships");

        assert_eq!(relationships.len(), 2);
        assert!(relationships.contains(&first));
        assert!(relationships.contains(&second));
    }

    #[tokio::test]
    async fn by_target_returns_relationships() {
        let pool = setup().await;
        let repository = RelationshipRepository::new(&pool);

        let target = EntityId::new();

        let first = Relationship::new(
            EntityId::new(),
            target,
            RelationshipKind::About,
        );

        let second = Relationship::new(
            EntityId::new(),
            target,
            RelationshipKind::References,
        );

        repository
            .insert(&first)
            .await
            .expect("failed to insert first relationship");

        repository
            .insert(&second)
            .await
            .expect("failed to insert second relationship");

        let relationships = repository
            .by_target(target)
            .await
            .expect("failed to query relationships");

        assert_eq!(relationships.len(), 2);
        assert!(relationships.contains(&first));
        assert!(relationships.contains(&second));
    }

    #[tokio::test]
    async fn duplicate_relationship_fails() {
        let pool = setup().await;
        let repository = RelationshipRepository::new(&pool);

        let relationship = Relationship::new(
            EntityId::new(),
            EntityId::new(),
            RelationshipKind::RelatedTo,
        );

        repository
            .insert(&relationship)
            .await
            .expect("failed to insert relationship");

        let result = repository.insert(&relationship).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_removes_relationship() {
        let pool = setup().await;
        let repository = RelationshipRepository::new(&pool);

        let relationship = Relationship::new(
            EntityId::new(),
            EntityId::new(),
            RelationshipKind::Uses,
        );

        repository
            .insert(&relationship)
            .await
            .expect("failed to insert relationship");

        assert!(
            repository
                .delete(
                    relationship.source,
                    relationship.target,
                    relationship.kind,
                )
                .await
                .expect("failed to delete relationship")
        );

        assert!(
            repository
                .get(
                    relationship.source,
                    relationship.target,
                    relationship.kind,
                )
                .await
                .expect("failed to get relationship")
                .is_none()
        );
    }

    #[tokio::test]
    async fn delete_unknown_relationship_returns_false() {
        let pool = setup().await;
        let repository = RelationshipRepository::new(&pool);

        assert!(
            !repository
                .delete(
                    EntityId::new(),
                    EntityId::new(),
                    RelationshipKind::About,
                )
                .await
                .expect("failed to delete relationship")
        );
    }
}