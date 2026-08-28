-- Core semantic relationships between entities.
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


-- Specific occurrences/references between entities.
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