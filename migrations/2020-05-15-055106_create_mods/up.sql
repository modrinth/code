CREATE TABLE mods (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    datePublished DATE NOT NULL,
    author VARCHAR NOT NULL,
    downloads INTEGER NOT NULL DEFAULT 0,
    categories TEXT[] NOT NULL,
    body_path VARCHAR NOT NULL,
    icon_path VARCHAR NOT NULL
)