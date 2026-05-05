CREATE TABLE featured_projects (
    user_id bigint REFERENCES users NOT NULL,
    mod_id bigint REFERENCES mods NOT NULL,
    created timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, mod_id)
);

CREATE INDEX idx_featured_projects_user_id ON featured_projects(user_id);
