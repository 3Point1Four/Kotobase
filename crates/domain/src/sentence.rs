use crate::EntityId;

/// A piece of Japanese text represented as a sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    pub id: EntityId,
    pub text: String,
    pub translation: Option<String>,
    pub source: Option<String>,
    pub tokens: Vec<Token>,
    pub grammar_matches: Vec<GrammarMatch>,
}

impl Sentence {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: EntityId::new(),
            text: text.into(),
            translation: None,
            source: None,
            tokens: Vec::new(),
            grammar_matches: Vec::new(),
        }
    }
}

/// A linguistically analyzed portion of a sentence.
///
/// `start` and `end` refer to character offsets in the sentence's original text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub id: EntityId,
    pub surface: String,
    pub start: usize,
    pub end: usize,
    pub reading: Option<String>,
    pub lemma: Option<String>,
    pub part_of_speech: Option<String>,
    pub vocabulary_id: Option<EntityId>,
}

impl Token {
    pub fn new(
        surface: impl Into<String>,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            id: EntityId::new(),
            surface: surface.into(),
            start,
            end,
            reading: None,
            lemma: None,
            part_of_speech: None,
            vocabulary_id: None,
        }
    }
}

/// A detected occurrence of a grammar pattern within a sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarMatch {
    pub id: EntityId,
    pub grammar_id: EntityId,
    pub start: usize,
    pub end: usize,
}

impl GrammarMatch {
    pub fn new(
        grammar_id: EntityId,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            id: EntityId::new(),
            grammar_id,
            start,
            end,
        }
    }
}