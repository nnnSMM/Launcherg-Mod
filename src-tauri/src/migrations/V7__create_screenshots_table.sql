CREATE TABLE screenshots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  game_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  tags TEXT, -- JSON array of strings
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (game_id) REFERENCES collection_elements(id) ON DELETE CASCADE
);

CREATE INDEX idx_screenshots_game_id ON screenshots(game_id);
