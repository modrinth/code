-- Core pairing. The backend only lists an account's Cores and where they are; the App talks to a
-- Core directly.

-- A code an unowned Core is showing. `claimed_by` is set when an account types it into the App.
CREATE TABLE core_pairing_codes (
  code TEXT PRIMARY KEY,
  core_id TEXT NOT NULL UNIQUE,
  address TEXT NOT NULL,
  expires TEXT NOT NULL,
  claimed_by TEXT REFERENCES users(id) ON DELETE CASCADE
);

-- A paired Core. `key_hash` is the SHA-256 of the key the Core uses for its own calls.
CREATE TABLE cores (
  id TEXT PRIMARY KEY,
  owner_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  address TEXT NOT NULL,
  key_hash TEXT NOT NULL UNIQUE,
  created TEXT NOT NULL,
  last_seen TEXT NOT NULL
);
CREATE INDEX cores_owner ON cores(owner_id);

-- Wrong pairing codes typed by an account. The count resets once `window_start` is old enough.
CREATE TABLE core_pairing_attempts (
  user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  failures INTEGER NOT NULL,
  window_start TEXT NOT NULL
);
