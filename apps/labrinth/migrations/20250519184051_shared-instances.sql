CREATE TABLE shared_instances (
  id BIGINT PRIMARY KEY,
  title VARCHAR(255) NOT NULL,
  owner_id BIGINT NOT NULL REFERENCES users,
  current_version_id BIGINT NULL,
  public BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX shared_instances_owner_id ON shared_instances(owner_id);

CREATE TABLE shared_instance_users (
  user_id BIGINT NOT NULL REFERENCES users ON DELETE CASCADE,
  shared_instance_id BIGINT NOT NULL REFERENCES shared_instances ON DELETE CASCADE,
  permissions BIGINT NOT NULL DEFAULT 0,

  PRIMARY KEY (user_id, shared_instance_id)
);

CREATE TABLE shared_instance_invited_users (
  id BIGINT PRIMARY KEY,
  shared_instance_id BIGINT NOT NULL REFERENCES shared_instances ON DELETE CASCADE,
  invited_user_id BIGINT NULL REFERENCES users ON DELETE CASCADE
);
CREATE INDEX shared_instance_invited_users_shared_instance_id ON shared_instance_invited_users(shared_instance_id);
CREATE INDEX shared_instance_invited_users_invited_user_id ON shared_instance_invited_users(invited_user_id);

CREATE TABLE shared_instance_invite_links (
  id BIGINT PRIMARY KEY,
  shared_instance_id BIGINT NOT NULL REFERENCES shared_instances ON DELETE CASCADE,
  expiration timestamptz NULL,
  remaining_uses BIGINT CHECK ( remaining_uses >= 0 ) NULL
);
CREATE INDEX shared_instance_invite_links_shared_instance_id ON shared_instance_invite_links(shared_instance_id);

CREATE TABLE shared_instance_versions (
  id BIGINT PRIMARY KEY,
  shared_instance_id BIGINT NOT NULL REFERENCES shared_instances ON DELETE CASCADE,
  size BIGINT NOT NULL,
  sha512 bytea NOT NULL,
  created timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE shared_instances
ADD FOREIGN KEY (current_version_id) REFERENCES shared_instance_versions(id) ON DELETE SET NULL;
