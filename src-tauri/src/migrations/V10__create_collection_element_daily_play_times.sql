CREATE TABLE IF NOT EXISTS collection_element_daily_play_times (
    collection_element_id INTEGER NOT NULL,
    play_date TEXT NOT NULL,
    play_time_seconds INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (collection_element_id, play_date),
    FOREIGN KEY (collection_element_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_collection_element_daily_play_times_play_date
ON collection_element_daily_play_times(play_date);
