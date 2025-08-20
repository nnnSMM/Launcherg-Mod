CREATE TRIGGER update_collection_elements_updated_at
AFTER UPDATE ON collection_elements
FOR EACH ROW
BEGIN
    UPDATE collection_elements
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.id;
END;
