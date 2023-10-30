CREATE TABLE oauth_clients (
    id bigint PRIMARY KEY,
    name text NOT NULL,
    icon_url text NULL,
    max_scopes bigint NOT NULL,
    secret_hash text NOT NULL,
    created timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint NOT NULL REFERENCES users(id)
);
CREATE TABLE oauth_client_redirect_uris (
    id bigint PRIMARY KEY,
    client_id bigint NOT NULL REFERENCES oauth_clients (id) ON DELETE CASCADE,
    uri text
);
CREATE TABLE oauth_client_authorizations (
    id bigint PRIMARY KEY,
    client_id bigint NOT NULL REFERENCES oauth_clients (id) ON DELETE CASCADE,
    user_id bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    scopes bigint NOT NULL,
    created timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (client_id, user_id)
);
CREATE TABLE oauth_access_tokens (
    id bigint PRIMARY KEY,
    authorization_id bigint NOT NULL REFERENCES oauth_client_authorizations(id) ON DELETE CASCADE,
    token_hash text NOT NULL UNIQUE,
    scopes bigint NOT NULL,
    created timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP + interval '14 days',
    last_used timestamptz NULL
);
CREATE INDEX oauth_client_creator ON oauth_clients(created_by);
CREATE INDEX oauth_redirect_client ON oauth_client_redirect_uris(client_id);
CREATE INDEX oauth_access_token_hash ON oauth_access_tokens(token_hash);