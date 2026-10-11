-- Pairing: the key the backend issued to this Core when its owner paired it.
-- An owner without a key is from before pairing existed and has to pair once to be listed.
ALTER TABLE core_owner ADD COLUMN backend_key TEXT;
