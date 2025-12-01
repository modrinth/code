ALTER TABLE mods
ADD COLUMN text_id TEXT NOT NULL DEFAULT '';

ALTER TABLE mods
ADD COLUMN text_id_lower TEXT GENERATED ALWAYS AS (lower(text_id)) STORED;

UPDATE mods
SET text_id = to_base62(id);

CREATE OR REPLACE FUNCTION update_text_id()
RETURNS TRIGGER AS $$
BEGIN
    NEW.text_id := to_base62(NEW.id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_text_id
    BEFORE INSERT OR UPDATE ON mods
    FOR EACH ROW EXECUTE FUNCTION update_text_id();

ALTER TABLE mods
ALTER COLUMN text_id SET NOT NULL;
