alter table dependencies
    add column dependency_sha1 bytea;

create table file_attributions (
	file_id bigint primary key references files(id),
	-- if a file..
	-- - does not have a row
	--   -> was created before attributions system
	-- - has a row, but `scanned_at = null`
	--   -> still needs to be scanned
	-- - has a row, and `scanned_at` is not null
	--   -> attributions have been scanned
	scanned_at timestamptz
);

create table project_attribution_groups (
	id bigint primary key,
	project_id bigint not null references mods(id),
	flame_project_id bigint,
	flame_project_title text,
	attribution jsonb
);
create index on project_attribution_groups (project_id);

create table project_attribution_files (
	group_id bigint not null references project_attribution_groups(id),
	name text not null,
	sha1 bytea not null,
	unique (sha1)
);
