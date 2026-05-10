CREATE TABLE IF NOT EXISTS vndb_screenshot_caches (
    collection_element_id INTEGER PRIMARY KEY,
    vndb_id TEXT,
    matched_title TEXT,
    screenshots_json TEXT NOT NULL DEFAULT '[]',
    fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL,
    FOREIGN KEY (collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);
