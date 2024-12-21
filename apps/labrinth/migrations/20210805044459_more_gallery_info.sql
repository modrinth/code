ALTER TABLE mods_gallery
    ADD COLUMN title varchar(255),
    ADD COLUMN description varchar(2048),
    ADD COLUMN created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL;


