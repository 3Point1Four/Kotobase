use domain::{
    EntityId,
    EntityType,
    Reference,
    ReferenceLocation,
};

pub struct ReferenceService;

impl ReferenceService {
    pub fn new() -> Self {
        Self
    }

    pub fn create(
        &self,
        source: EntityId,
        source_type: EntityType,
        target: EntityId,
        target_type: EntityType,
        location: ReferenceLocation,
    ) -> Reference {
        Reference::new(
            source,
            source_type,
            target,
            target_type,
            location,
        )
    }

    pub fn source_of(
        &self,
        reference: &Reference,
    ) -> EntityId {
        reference.source
    }

    pub fn target_of(
        &self,
        reference: &Reference,
    ) -> EntityId {
        reference.target
    }
}

impl Default for ReferenceService {
    fn default() -> Self {
        Self::new()
    }
}