ALTER TABLE threads
DROP CONSTRAINT threads_mod_id_fkey,
ADD CONSTRAINT threads_mod_id_fkey
FOREIGN KEY (mod_id) REFERENCES mods(id)
ON DELETE SET NULL;
