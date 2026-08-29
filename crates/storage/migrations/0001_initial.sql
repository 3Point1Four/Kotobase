-- ============================================================
-- Kotobase initial database schema
-- ============================================================

-- ============================================================
-- Vocabulary
-- ============================================================

CREATE TABLE vocabulary (
    id TEXT PRIMARY KEY,
    source TEXT
);

CREATE TABLE vocabulary_written_forms (
    vocabulary_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    form TEXT NOT NULL,

    PRIMARY KEY (vocabulary_id, position),

    FOREIGN KEY (vocabulary_id)
        REFERENCES vocabulary(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_vocabulary_written_forms_form
    ON vocabulary_written_forms (form);

CREATE INDEX idx_vocabulary_written_forms_vocabulary
    ON vocabulary_written_forms (vocabulary_id);


CREATE TABLE vocabulary_readings (
    vocabulary_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    reading TEXT NOT NULL,

    PRIMARY KEY (vocabulary_id, position),

    FOREIGN KEY (vocabulary_id)
        REFERENCES vocabulary(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_vocabulary_readings_reading
    ON vocabulary_readings (reading);

CREATE INDEX idx_vocabulary_readings_vocabulary
    ON vocabulary_readings (vocabulary_id);


CREATE TABLE vocabulary_meanings (
    vocabulary_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    meaning TEXT NOT NULL,

    PRIMARY KEY (vocabulary_id, position),

    FOREIGN KEY (vocabulary_id)
        REFERENCES vocabulary(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_vocabulary_meanings_meaning
    ON vocabulary_meanings (meaning);

CREATE INDEX idx_vocabulary_meanings_vocabulary
    ON vocabulary_meanings (vocabulary_id);


CREATE TABLE vocabulary_parts_of_speech (
    vocabulary_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    part_of_speech TEXT NOT NULL,

    PRIMARY KEY (vocabulary_id, position),

    FOREIGN KEY (vocabulary_id)
        REFERENCES vocabulary(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_vocabulary_parts_of_speech_vocabulary
    ON vocabulary_parts_of_speech (vocabulary_id);

CREATE INDEX idx_vocabulary_parts_of_speech_pos
    ON vocabulary_parts_of_speech (part_of_speech);


-- ============================================================
-- Semantic relationships between entities
-- ============================================================

CREATE TABLE relationships (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    label TEXT,
    context TEXT,

    PRIMARY KEY (source_id, target_id, kind)
);

CREATE INDEX idx_relationships_source
    ON relationships (source_id);

CREATE INDEX idx_relationships_target
    ON relationships (target_id);

CREATE INDEX idx_relationships_source_kind
    ON relationships (source_id, kind);

CREATE INDEX idx_relationships_target_kind
    ON relationships (target_id, kind);


-- ============================================================
-- Specific occurrences/references between entities
-- ============================================================

CREATE TABLE entity_references (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    source_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    target_type TEXT NOT NULL,
    location_type TEXT NOT NULL,
    start_offset INTEGER,
    end_offset INTEGER,
    page INTEGER,
    block_id TEXT
);

CREATE INDEX idx_entity_references_source
    ON entity_references (source_id);

CREATE INDEX idx_entity_references_target
    ON entity_references (target_id);

CREATE INDEX idx_entity_references_source_type
    ON entity_references (source_id, source_type);

CREATE INDEX idx_entity_references_target_type
    ON entity_references (target_id, target_type);