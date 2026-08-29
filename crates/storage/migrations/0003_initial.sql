-- ============================================================
-- Grammar patterns
-- ============================================================

CREATE TABLE grammar_patterns (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    formation TEXT NOT NULL,
    usage TEXT NOT NULL,
    source TEXT
);

CREATE TABLE grammar_pattern_meanings (
    grammar_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    meaning TEXT NOT NULL,
    PRIMARY KEY (grammar_id, position),
    FOREIGN KEY (grammar_id)
        REFERENCES grammar_patterns(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_grammar_pattern_meanings_meaning
    ON grammar_pattern_meanings (meaning);

CREATE INDEX idx_grammar_pattern_meanings_grammar
    ON grammar_pattern_meanings (grammar_id);

CREATE INDEX idx_grammar_patterns_name
    ON grammar_patterns (name);