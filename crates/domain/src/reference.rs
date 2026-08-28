use crate::EntityId;

/// Identifies the kind of entity being referenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Vocabulary,
    Kanji,
    Grammar,
    Sentence,
    Note,
    Page,
    File,
    Tag,
    Meaning,
    Proverb,
    StudySession,
}

/// Identifies where an entity is referenced within another entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub id: EntityId,
    pub target: EntityId,
    pub target_type: EntityType,
    pub source: EntityId,
    pub source_type: EntityType,
    pub location: ReferenceLocation,
}

/// Describes the precise location of a reference within its source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceLocation {
    /// A range of characters within text.
    CharacterRange {
        start: usize,
        end: usize,
    },

    /// A range of analyzed tokens.
    TokenRange {
        start: usize,
        end: usize,
    },

    /// A page within a paginated document.
    Page {
        page: u32,
    },

    /// A specific content block.
    Block {
        block_id: EntityId,
    },

    /// A location within an external file.
    FileLocation {
        page: Option<u32>,
        start: Option<usize>,
        end: Option<usize>,
    },
}

impl Reference {
    pub fn new(
        source: EntityId,
        source_type: EntityType,
        target: EntityId,
        target_type: EntityType,
        location: ReferenceLocation,
    ) -> Self {
        Self {
            id: EntityId::new(),
            source,
            source_type,
            target,
            target_type,
            location,
        }
    }
}