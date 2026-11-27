-- Bitfield exposed by the API as collection_flags.
-- 1 = featured, 2 = potw.
ALTER TABLE collections
    ADD COLUMN flags bigint NOT NULL DEFAULT 0;

CREATE INDEX collections_mods_mod_id_idx
    ON collections_mods (mod_id, collection_id);
