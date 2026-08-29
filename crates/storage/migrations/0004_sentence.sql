-- ============================================================
-- Sentences
-- ============================================================

CREATE TABLE sentences (
    id TEXT PRIMARY KEY,
    text TEXT NOT NULL,
    translation TEXT,
    source TEXT
);

CREATE TABLE sentence_tokens (
    id TEXT PRIMARY KEY,
    sentence_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    surface TEXT NOT NULL,
    start_offset INTEGER NOT NULL,
    end_offset INTEGER NOT NULL,
    reading TEXT,
    lemma TEXT,
    part_of_speech TEXT,
    vocabulary_id TEXT,
    FOREIGN KEY (sentence_id)
        REFERENCES sentences(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_sentence_tokens_sentence
    ON sentence_tokens (sentence_id);

CREATE INDEX idx_sentence_tokens_vocabulary
    ON sentence_tokens (vocabulary_id);

CREATE TABLE sentence_grammar_matches (
    id TEXT PRIMARY KEY,
    sentence_id TEXT NOT NULL,
    grammar_id TEXT NOT NULL,
    start_offset INTEGER NOT NULL,
    end_offset INTEGER NOT NULL,
    FOREIGN KEY (sentence_id)
        REFERENCES sentences(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_sentence_grammar_matches_sentence
    ON sentence_grammar_matches (sentence_id);

CREATE INDEX idx_sentence_grammar_matches_grammar
    ON sentence_grammar_matches (grammar_id);