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