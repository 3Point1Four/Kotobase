use crate::EntityId;

/// Identifies where an entity is referenced within another entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub id: EntityId,
    pub target: EntityId,
    pub source: EntityId,
    pub location: ReferenceLocation,
}

/// Describes the location of a reference within its source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceLocation {
    CharacterRange {
        start: usize,
        end: usize,
    },
    TokenRange {
        start: usize,
        end: usize,
    },
    Page {
        page: u32,
    },
    Block {
        block_id: EntityId,
    },
}