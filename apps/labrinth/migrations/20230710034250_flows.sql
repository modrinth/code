CREATE INDEX sessions_session
    ON sessions (session);

CREATE TABLE flows (
  id bigint NOT NULL PRIMARY KEY,
  flow varchar(64) NOT NULL UNIQUE,
  user_id BIGINT NOT NULL REFERENCES users(id),
  expires timestamptz NOT NULL,
  flow_type varchar(64) NOT NULL
);

CREATE INDEX flows_flow
    ON flows (flow);

DROP TABLE pats;

CREATE TABLE pats (
  id BIGINT PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  user_id BIGINT NOT NULL REFERENCES users(id),
  access_token VARCHAR(64) NOT NULL UNIQUE,
  scopes BIGINT NOT NULL,
  created timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires timestamptz NOT NULL,
  last_used timestamptz NULL
);

CREATE INDEX pats_user_id
    ON pats (user_id);

CREATE INDEX pats_access_token
    ON pats (access_token);

ALTER TABLE mods DROP COLUMN thread_id;
ALTER TABLE reports DROP COLUMN thread_id;

DELETE FROM threads_members;
DELETE FROM threads_messages;
DELETE FROM threads;

ALTER TABLE threads
    ADD COLUMN report_id bigint references reports ON UPDATE CASCADE NULL;
ALTER TABLE threads
    ADD COLUMN mod_id bigint references mods ON UPDATE CASCADE NULL;