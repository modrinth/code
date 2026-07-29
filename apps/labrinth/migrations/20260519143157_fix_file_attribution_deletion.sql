alter table file_scans
	drop constraint file_scans_file_id_fkey,
	add constraint file_scans_file_id_fkey
		foreign key (file_id) references files(id) on delete cascade;

alter table project_attribution_groups
	drop constraint project_attribution_groups_project_id_fkey,
	add constraint project_attribution_groups_project_id_fkey
		foreign key (project_id) references mods(id) on delete cascade;

alter table project_attribution_files
	drop constraint project_attribution_files_group_id_fkey,
	add constraint project_attribution_files_group_id_fkey
		foreign key (group_id) references project_attribution_groups(id) on delete cascade;

alter table override_file_sources
	drop constraint override_file_sources_file_id_fkey,
	add constraint override_file_sources_file_id_fkey
		foreign key (file_id) references files(id) on delete cascade;
