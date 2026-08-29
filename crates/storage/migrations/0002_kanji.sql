-- ============================================================
-- Kanji
-- ============================================================

CREATE TABLE kanji (
    id TEXT PRIMARY KEY,
    character TEXT NOT NULL UNIQUE,
    stroke_count INTEGER,
    grade INTEGER,
    jlpt_level INTEGER
);

CREATE TABLE kanji_on_readings (
    kanji_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    reading TEXT NOT NULL,
    PRIMARY KEY (kanji_id, position),
    FOREIGN KEY (kanji_id)
        REFERENCES kanji(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_kanji_on_readings_reading
    ON kanji_on_readings (reading);

CREATE INDEX idx_kanji_on_readings_kanji
    ON kanji_on_readings (kanji_id);

CREATE TABLE kanji_kun_readings (
    kanji_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    reading TEXT NOT NULL,
    PRIMARY KEY (kanji_id, position),
    FOREIGN KEY (kanji_id)
        REFERENCES kanji(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_kanji_kun_readings_reading
    ON kanji_kun_readings (reading);

CREATE INDEX idx_kanji_kun_readings_kanji
    ON kanji_kun_readings (kanji_id);

CREATE TABLE kanji_meanings (
    kanji_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    meaning TEXT NOT NULL,
    PRIMARY KEY (kanji_id, position),
    FOREIGN KEY (kanji_id)
        REFERENCES kanji(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_kanji_meanings_meaning
    ON kanji_meanings (meaning);

CREATE INDEX idx_kanji_meanings_kanji
    ON kanji_meanings (kanji_id);