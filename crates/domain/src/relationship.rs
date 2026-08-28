use crate::EntityId;

/// Describes how two entities are related.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipKind {
    About,
    Contains,
    Realizes,
    Uses,
    References,
    RelatedTo,
}

/// A directed relationship between two entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Relationship {
    pub source: EntityId,
    pub target: EntityId,
    pub kind: RelationshipKind,
}

impl Relationship {
    pub fn new(source: EntityId, target: EntityId, kind: RelationshipKind) -> Self {
        Self {
            source,
            target,
            kind,
        }
    }
}