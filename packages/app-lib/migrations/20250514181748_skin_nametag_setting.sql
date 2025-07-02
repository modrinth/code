ALTER TABLE settings ADD COLUMN hide_nametag_skins_page INTEGER NOT NULL DEFAULT 0 CHECK (hide_nametag_skins_page IN (0, 1));
