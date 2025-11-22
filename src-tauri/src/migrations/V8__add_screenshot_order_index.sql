-- Add order_index column and remove tags column from screenshots table

-- Add order_index column
ALTER TABLE screenshots ADD COLUMN order_index INTEGER NOT NULL DEFAULT 0;

-- Set order_index based on created_at for existing records
UPDATE screenshots SET order_index = (
    SELECT COUNT(*) FROM screenshots AS s2 
    WHERE s2.game_id = screenshots.game_id 
    AND s2.created_at < screenshots.created_at
);

-- Remove tags column
ALTER TABLE screenshots DROP COLUMN tags;
