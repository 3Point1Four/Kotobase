use domain::{
    EntityId,
    Relationship,
    RelationshipKind,
};
use storage::repositories::RelationshipRepository;

pub struct RelationshipService<'a> {
    repository: RelationshipRepository<'a>,
}

impl<'a> RelationshipService<'a> {
    pub fn new(repository: RelationshipRepository<'a>) -> Self {
        Self { repository }
    }

    pub async fn create(
        &self,
        source: EntityId,
        target: EntityId,
        kind: RelationshipKind,
    ) -> Result<Relationship, sqlx::Error> {
        let relationship = Relationship::new(
            source,
            target,
            kind,
        );

        self.repository.insert(&relationship).await?;

        Ok(relationship)
    }

    pub async fn get(
        &self,
        source: EntityId,
        target: EntityId,
        kind: RelationshipKind,
    ) -> Result<Option<Relationship>, sqlx::Error> {
        self.repository
            .get(source, target, kind)
            .await
    }

    pub async fn by_source(
        &self,
        source: EntityId,
    ) -> Result<Vec<Relationship>, sqlx::Error> {
        self.repository.by_source(source).await
    }

    pub async fn by_target(
        &self,
        target: EntityId,
    ) -> Result<Vec<Relationship>, sqlx::Error> {
        self.repository.by_target(target).await
    }

    pub async fn delete(
        &self,
        source: EntityId,
        target: EntityId,
        kind: RelationshipKind,
    ) -> Result<bool, sqlx::Error> {
        self.repository
            .delete(source, target, kind)
            .await
    }
}