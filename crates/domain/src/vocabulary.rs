use crate::EntityId;

/// A Japanese vocabulary entry representing a lexical concept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VocabularyEntry {
    pub id: EntityId,
    pub written_forms: Vec<String>,
    pub readings: Vec<String>,
    pub meanings: Vec<String>,
    pub parts_of_speech: Vec<PartOfSpeech>,
    pub source: Option<String>,
}

impl VocabularyEntry {
    pub fn new() -> Self {
        Self {
            id: EntityId::new(),
            written_forms: Vec::new(),
            readings: Vec::new(),
            meanings: Vec::new(),
            parts_of_speech: Vec::new(),
            source: None,
        }
    }
}

impl Default for VocabularyEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// A broad grammatical classification for a vocabulary entry.
///
/// This will grow as the Japanese analysis system becomes more sophisticated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    IAdjective,
    NaAdjective,
    Adverb,
    Particle,
    Auxiliary,
    Conjunction,
    Interjection,
    Pronoun,
    Determiner,
    Counter,
    Prefix,
    Suffix,
    Other,
}