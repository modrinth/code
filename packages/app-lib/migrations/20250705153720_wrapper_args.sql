ALTER TABLE profiles ADD COLUMN override_hook_wrapper_type TEXT NOT NULL DEFAULT 'path';
ALTER TABLE settings ADD COLUMN hook_wrapper_type TEXT NOT NULL DEFAULT 'path';
