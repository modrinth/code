ALTER TABLE profiles
ADD COLUMN preferred_update_channel TEXT NOT NULL DEFAULT 'release';
