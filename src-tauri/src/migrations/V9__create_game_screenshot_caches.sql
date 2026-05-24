CREATE TABLE IF NOT EXISTS game_screenshot_caches (
    collection_element_id INTEGER PRIMARY KEY,
    matched_title TEXT,
    screenshots_json TEXT NOT NULL DEFAULT '[]',
    fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL,
    FOREIGN KEY (collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);
