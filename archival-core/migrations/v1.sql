-- Archival Schema v1
-- All primary keys are UUID TEXT. All timestamps are Unix seconds (INTEGER).
-- Media files are stored as flat files; DB holds relative paths only.

CREATE TABLE items (
    id          TEXT    NOT NULL PRIMARY KEY,
    category    TEXT    NOT NULL,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    notes       TEXT,
    CONSTRAINT category_valid CHECK (
        category IN ('Book','Music','Movie','Game','PersonalMessage',
                     'Award','Art','Photograph','Trinket','Jewelry',
                     'Clothing','Object')
    )
);

CREATE TABLE media_files (
    id          TEXT    NOT NULL PRIMARY KEY,
    item_id     TEXT    NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    file_path   TEXT    NOT NULL UNIQUE,
    mime_type   TEXT    NOT NULL,
    is_primary  INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL
);

CREATE TABLE tags (
    id   TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE item_tags (
    item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    tag_id  TEXT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (item_id, tag_id)
);

CREATE TABLE collections (
    id          TEXT    NOT NULL PRIMARY KEY,
    name        TEXT    NOT NULL UNIQUE,
    description TEXT,
    created_at  INTEGER NOT NULL
);

CREATE TABLE collection_items (
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    item_id       TEXT NOT NULL REFERENCES items(id)       ON DELETE CASCADE,
    PRIMARY KEY (collection_id, item_id)
);

-- Category-specific field tables (one per category)

CREATE TABLE book_fields (
    item_id         TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    title           TEXT,
    author          TEXT,
    isbn            TEXT,
    dewey_decimal   TEXT,
    pub_date        TEXT,
    pub_city        TEXT,
    pub_country     TEXT,
    pub_language    TEXT,
    summary         TEXT
);

CREATE TABLE music_fields (
    item_id  TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    title    TEXT,
    artist   TEXT,
    album    TEXT,
    year     TEXT,
    label    TEXT,
    genre    TEXT,
    format   TEXT
);

CREATE TABLE movie_fields (
    item_id   TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    title     TEXT,
    director  TEXT,
    year      TEXT,
    studio    TEXT,
    format    TEXT,
    language  TEXT,
    synopsis  TEXT
);

CREATE TABLE game_fields (
    item_id    TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    title      TEXT,
    developer  TEXT,
    publisher  TEXT,
    year       TEXT,
    platform   TEXT,
    genre      TEXT
);

CREATE TABLE personal_message_fields (
    item_id    TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    sender     TEXT,
    recipient  TEXT,
    date_sent  TEXT,
    medium     TEXT,
    occasion   TEXT
);

CREATE TABLE award_fields (
    item_id        TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    title          TEXT,
    issuing_org    TEXT,
    date_received  TEXT,
    category       TEXT,
    description    TEXT
);

CREATE TABLE art_fields (
    item_id     TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    title       TEXT,
    artist      TEXT,
    medium      TEXT,
    year        TEXT,
    dimensions  TEXT,
    provenance  TEXT
);

CREATE TABLE photograph_fields (
    item_id       TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    subject       TEXT,
    date_taken    TEXT,
    location      TEXT,
    photographer  TEXT,
    format        TEXT
);

CREATE TABLE trinket_fields (
    item_id          TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    name             TEXT,
    origin           TEXT,
    material         TEXT,
    approximate_date TEXT
);

CREATE TABLE jewelry_fields (
    item_id          TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    name             TEXT,
    material         TEXT,
    gemstones        TEXT,
    maker            TEXT,
    approximate_date TEXT
);

CREATE TABLE clothing_fields (
    item_id          TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    name             TEXT,
    brand            TEXT,
    size             TEXT,
    material         TEXT,
    color            TEXT,
    approximate_date TEXT
);

CREATE TABLE object_fields (
    item_id          TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    name             TEXT,
    material         TEXT,
    manufacturer     TEXT,
    approximate_date TEXT,
    description      TEXT
);

-- Indexes for common access patterns
CREATE INDEX idx_items_category   ON items(category);
CREATE INDEX idx_items_created    ON items(created_at);
CREATE INDEX idx_media_item       ON media_files(item_id);
CREATE INDEX idx_item_tags_item   ON item_tags(item_id);
CREATE INDEX idx_item_tags_tag    ON item_tags(tag_id);
CREATE INDEX idx_coll_items       ON collection_items(collection_id);

-- Schema version tracking
CREATE TABLE schema_version (
    version    INTEGER NOT NULL,
    applied_at INTEGER NOT NULL
);
INSERT INTO schema_version VALUES (1, strftime('%s', 'now'));
