use crate::EntityId;

/// A single Japanese kanji character and its linguistic information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KanjiEntry {
    pub id: EntityId,
    pub character: char,
    pub on_readings: Vec<String>,
    pub kun_readings: Vec<String>,
    pub meanings: Vec<String>,
    pub stroke_count: Option<u16>,
    pub grade: Option<u8>,
    pub jlpt_level: Option<u8>,
}

impl KanjiEntry {
    pub fn new(character: char) -> Self {
        Self {
            id: EntityId::new(),
            character,
            on_readings: Vec::new(),
            kun_readings: Vec::new(),
            meanings: Vec::new(),
            stroke_count: None,
            grade: None,
            jlpt_level: None,
        }
    }
}