use domain::{EntityId, VocabularyEntry};

pub struct VocabularyService;

impl VocabularyService {
    pub fn new() -> Self {
        Self
    }

    pub fn create(&self) -> VocabularyEntry {
        VocabularyEntry::new()
    }

    pub fn identify(
        &self,
        vocabulary: &VocabularyEntry,
    ) -> EntityId {
        vocabulary.id
    }
}

impl Default for VocabularyService {
    fn default() -> Self {
        Self::new()
    }
}