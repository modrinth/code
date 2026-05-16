CREATE TABLE project_reviews (
    id BIGINT PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    body VARCHAR(8192) NOT NULL DEFAULT '',
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (project_id, user_id)
);

CREATE INDEX project_reviews_project_id_idx ON project_reviews (project_id);
CREATE INDEX project_reviews_user_id_idx ON project_reviews (user_id);
