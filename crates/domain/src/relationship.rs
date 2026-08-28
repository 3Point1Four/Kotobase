use crate::EntityId;

/// Describes the semantic relationship between two entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationshipKind {
    /// The source is about the target.
    About,

    /// The source contains the target.
    Contains,

    /// The source references the target.
    References,

    /// The source uses the target.
    Uses,

    /// The source is an example of the target.
    ExampleOf,

    /// The source explains the target.
    Explains,

    /// The source translates the target.
    Translates,

    /// The source was derived from the target.
    DerivedFrom,

    /// The source is related to the target without a more specific relationship.
    RelatedTo,
}

/// Optional metadata describing why or how a relationship exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationshipMetadata {
    pub label: Option<String>,
    pub context: Option<String>,
}

impl RelationshipMetadata {
    pub fn new() -> Self {
        Self {
            label: None,
            context: None,
        }
    }
}

impl Default for RelationshipMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// A directed semantic relationship between two entities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    pub source: EntityId,
    pub target: EntityId,
    pub kind: RelationshipKind,
    pub metadata: RelationshipMetadata,
}

impl Relationship {
    pub fn new(
        source: EntityId,
        target: EntityId,
        kind: RelationshipKind,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            metadata: RelationshipMetadata::default(),
        }
    }

    pub fn with_metadata(
        source: EntityId,
        target: EntityId,
        kind: RelationshipKind,
        metadata: RelationshipMetadata,
    ) -> Self {
        Self {
            source,
            target,
            kind,
            metadata,
        }
    }
}