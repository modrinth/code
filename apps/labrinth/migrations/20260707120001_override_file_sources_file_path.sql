alter table override_file_sources
	add column file_path text not null default '',
	drop constraint override_file_sources_pkey,
	add primary key (sha1, file_id, file_path);
