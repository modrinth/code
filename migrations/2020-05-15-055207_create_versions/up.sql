CREATE TABLE versions (
    version_id SERIAL PRIMARY KEY,
    mod_id SERIAL NOT NULL,
    title VARCHAR NOT NULL,
    changelog_path VARCHAR NOT NULL,
    files_path TEXT[] NOT NULL,
    date_published DATE NOT NULL,
    author VARCHAR NOT NULL,
    downloads INTEGER NOT NULL DEFAULT 0,
    dependencies TEXT[] NOT NULL,
    game_versions TEXT[] NOT NULL
)