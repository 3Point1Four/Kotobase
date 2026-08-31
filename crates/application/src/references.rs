use domain::{
    EntityId,
    EntityType,
    Reference,
    ReferenceLocation,
};
use storage::repositories::ReferenceRepository;

pub struct ReferenceService<'a> {
    repository: ReferenceRepository<'a>,
}

impl<'a> ReferenceService<'a> {
    pub fn new(repository: ReferenceRepository<'a>) -> Self {
        Self { repository }
    }

    pub async fn create(
        &self,
        source: EntityId,
        source_type: EntityType,
        target: EntityId,
        target_type: EntityType,
        location: ReferenceLocation,
    ) -> Result<Reference, sqlx::Error> {
        let reference = Reference::new(
            source,
            source_type,
            target,
            target_type,
            location,
        );

        self.repository.insert(&reference).await?;

        Ok(reference)
    }

    pub async fn get(
        &self,
        id: EntityId,
    ) -> Result<Option<Reference>, sqlx::Error> {
        self.repository.get(id).await
    }

    pub async fn by_source(
        &self,
        source: EntityId,
    ) -> Result<Vec<Reference>, sqlx::Error> {
        self.repository.by_source(source).await
    }

    pub async fn by_target(
        &self,
        target: EntityId,
    ) -> Result<Vec<Reference>, sqlx::Error> {
        self.repository.by_target(target).await
    }

    pub async fn delete(
        &self,
        id: EntityId,
    ) -> Result<bool, sqlx::Error> {
        self.repository.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .expect("failed to create test database");

        storage::initialize_database(&pool)
            .await
            .expect("failed to run migrations");

        pool
    }

    fn sample_location() -> ReferenceLocation {
    ReferenceLocation::FileLocation {
        page: Some(2),
        start: Some(10),
        end: Some(14),
    }
}

    #[tokio::test]
    async fn create_and_get_reference() {
        let pool = setup().await;
        let repository = ReferenceRepository::new(&pool);
        let service = ReferenceService::new(repository);

        let source = EntityId::new();
        let target = EntityId::new();

        let reference = service
            .create(
                source,
                EntityType::Vocabulary,
                target,
                EntityType::Kanji,
                sample_location(),
            )
            .await
            .expect("failed to create reference");

        let loaded = service
            .get(reference.id)
            .await
            .expect("failed to get reference")
            .expect("reference was not found");

        assert_eq!(loaded, reference);
    }

    #[tokio::test]
    async fn by_source_finds_reference() {
        let pool = setup().await;
        let repository = ReferenceRepository::new(&pool);
        let service = ReferenceService::new(repository);

        let source = EntityId::new();
        let target = EntityId::new();

        let reference = service
            .create(
                source,
                EntityType::Vocabulary,
                target,
                EntityType::Kanji,
                sample_location(),
            )
            .await
            .expect("failed to create reference");

        let references = service
            .by_source(source)
            .await
            .expect("failed to query references");

        assert_eq!(references.len(), 1);
        assert_eq!(references[0], reference);
    }

    #[tokio::test]
    async fn by_target_finds_reference() {
        let pool = setup().await;
        let repository = ReferenceRepository::new(&pool);
        let service = ReferenceService::new(repository);

        let source = EntityId::new();
        let target = EntityId::new();

        let reference = service
            .create(
                source,
                EntityType::Vocabulary,
                target,
                EntityType::Kanji,
                sample_location(),
            )
            .await
            .expect("failed to create reference");

        let references = service
            .by_target(target)
            .await
            .expect("failed to query references");

        assert_eq!(references.len(), 1);
        assert_eq!(references[0], reference);
    }

    #[tokio::test]
    async fn by_source_returns_empty_for_unknown_entity() {
        let pool = setup().await;
        let repository = ReferenceRepository::new(&pool);
        let service = ReferenceService::new(repository);

        let references = service
            .by_source(EntityId::new())
            .await
            .expect("failed to query references");

        assert!(references.is_empty());
    }

    #[tokio::test]
    async fn by_target_returns_empty_for_unknown_entity() {
        let pool = setup().await;
        let repository = ReferenceRepository::new(&pool);
        let service = ReferenceService::new(repository);

        let references = service
            .by_target(EntityId::new())
            .await
            .expect("failed to query references");

        assert!(references.is_empty());
    }

    #[tokio::test]
    async fn get_returns_none_for_unknown_reference() {
        let pool = setup().await;
        let repository = ReferenceRepository::new(&pool);
        let service = ReferenceService::new(repository);

        let result = service
            .get(EntityId::new())
            .await
            .expect("failed to get reference");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn delete_removes_reference() {
        let pool = setup().await;
        let repository = ReferenceRepository::new(&pool);
        let service = ReferenceService::new(repository);

        let reference = service
            .create(
                EntityId::new(),
                EntityType::Vocabulary,
                EntityId::new(),
                EntityType::Kanji,
                sample_location(),
            )
            .await
            .expect("failed to create reference");

        assert!(
            service
                .delete(reference.id)
                .await
                .expect("failed to delete reference")
        );

        assert!(
            service
                .get(reference.id)
                .await
                .expect("failed to get reference")
                .is_none()
        );
    }

    #[tokio::test]
    async fn deleting_unknown_reference_returns_false() {
        let pool = setup().await;
        let repository = ReferenceRepository::new(&pool);
        let service = ReferenceService::new(repository);

        assert!(
            !service
                .delete(EntityId::new())
                .await
                .expect("failed to delete reference")
        );
    }
}