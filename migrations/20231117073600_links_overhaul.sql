CREATE TABLE link_platforms (
    id SERIAL PRIMARY KEY,
    name VARCHAR(16) UNIQUE NOT NULL,

    -- Used for v2 conversion
    donation BOOLEAN NOT NULL DEFAULT false,
    -- Will be removed at the end of the migration
    donation_platform_id INTEGER REFERENCES donation_platforms (id)
);

INSERT INTO link_platforms (donation_platform_id, name, donation) 
SELECT id, short, true FROM donation_platforms;

INSERT INTO link_platforms (name, donation) VALUES ('issues', false);
INSERT INTO link_platforms (name, donation) VALUES ('wiki',  false);
INSERT INTO link_platforms (name, donation) VALUES ('discord', false);
INSERT INTO link_platforms (name, donation) VALUES ('source',  false);
INSERT INTO link_platforms (name, donation) VALUES ('site',  false);

CREATE TABLE mods_links (
    id SERIAL PRIMARY KEY,
    joining_mod_id BIGINT NOT NULL REFERENCES mods (id), 
    joining_platform_id INTEGER NOT NULL REFERENCES link_platforms (id),
    url VARCHAR(2048) NOT NULL
);

INSERT INTO mods_links (joining_mod_id, joining_platform_id, url)
SELECT DISTINCT m.id, lp.id, md.url
FROM mods m
INNER JOIN mods_donations md ON m.id = md.joining_mod_id
INNER JOIN donation_platforms dp ON dp.id = md.joining_platform_id
INNER JOIN link_platforms lp ON lp.donation_platform_id = dp.id; 

INSERT INTO mods_links (joining_mod_id, joining_platform_id, url) 
SELECT DISTINCT m.id, lp.id, issues_url 
FROM mods m
CROSS JOIN link_platforms lp 
WHERE issues_url IS NOT NULL
AND lp.name = 'issues';

INSERT INTO mods_links (joining_mod_id, joining_platform_id, url)
SELECT DISTINCT m.id, lp.id, wiki_url
FROM mods m
CROSS JOIN link_platforms lp
WHERE wiki_url IS NOT NULL
AND lp.name = 'wiki';

INSERT INTO mods_links (joining_mod_id, joining_platform_id, url)
SELECT DISTINCT m.id, lp.id, discord_url
FROM mods m
CROSS JOIN link_platforms lp
WHERE discord_url IS NOT NULL
AND lp.name = 'discord';

INSERT INTO mods_links (joining_mod_id, joining_platform_id, url)
SELECT DISTINCT m.id, lp.id, source_url
FROM mods m
CROSS JOIN link_platforms lp
WHERE source_url IS NOT NULL
AND lp.name = 'source';

ALTER TABLE mods DROP COLUMN issues_url;
ALTER TABLE mods DROP COLUMN wiki_url;
ALTER TABLE mods DROP COLUMN discord_url;
ALTER TABLE mods DROP COLUMN source_url;

ALTER TABLE link_platforms DROP COLUMN donation_platform_id;
DROP TABLE mods_donations;
DROP TABLE donation_platforms;