create table file_scans (
	file_id bigint primary key references files(id),
	-- if a file..
	-- - does not have a row
	--   -> was created before attributions system
	-- - has a row, but `attributions_scanned_at = null`
	--   -> still needs to be scanned
	-- - has a row, and `attributions_scanned_at` is not null
	--   -> attributions have been scanned
	attributions_scanned_at timestamptz
);

create table project_attribution_groups (
	id bigint primary key,
	project_id bigint not null references mods(id),
	flame_project jsonb,
	attribution jsonb,
	attributed_at timestamptz,
	attributed_by bigint references users(id)
);
create index on project_attribution_groups (project_id);

create table project_attribution_files (
	group_id bigint not null references project_attribution_groups(id),
	name text not null,
	sha1 bytea not null
);

create table override_file_sources (
	sha1 bytea not null,
	file_id bigint not null references files(id),
	primary key (sha1, file_id)
);
