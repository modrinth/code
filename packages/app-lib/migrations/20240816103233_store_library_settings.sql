ALTER TABLE settings
  ADD library_sort   TEXT NOT NULL DEFAULT 'Name';
ALTER TABLE settings
  ADD library_filter TEXT NOT NULL DEFAULT 'All profiles';
ALTER TABLE settings
  ADD library_group  TEXT NOT NULL DEFAULT 'Category';
