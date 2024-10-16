-- Add migration script here
ALTER TABLE states ADD COLUMN user_id bigint references users ON UPDATE CASCADE NULL;

ALTER TABLE users ADD COLUMN totp_secret varchar(24) null;

ALTER TABLE users ADD CONSTRAINT email_unique UNIQUE (email);

DROP TABLE flows;
DROP TABLE states;

CREATE TABLE user_backup_codes (
   user_id BIGINT NOT NULL REFERENCES users(id),
   code BIGINT NOT NULL,
   PRIMARY KEY (user_id, code)
);