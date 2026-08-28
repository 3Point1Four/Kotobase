use crate::EntityId;

/// A Japanese grammar pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarPattern {
    pub id: EntityId,
    pub name: String,
    pub formation: String,
    pub meanings: Vec<String>,
    pub usage: String,
    pub source: Option<String>,
}

impl GrammarPattern {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: EntityId::new(),
            name: name.into(),
            formation: String::new(),
            meanings: Vec::new(),
            usage: String::new(),
            source: None,
        }
    }
}