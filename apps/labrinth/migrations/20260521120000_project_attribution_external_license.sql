alter table project_attribution_files
	add column moderation_external_license_id bigint references moderation_external_licenses(id);

create table attributions_exemptions (
	version_id bigint references versions(id) on delete cascade,
	project_id bigint references mods(id) on delete cascade,
	check ((version_id is null) != (project_id is null))
);

create unique index attributions_exemptions_version_id_idx
	on attributions_exemptions (version_id)
	where version_id is not null;

create unique index attributions_exemptions_project_id_idx
	on attributions_exemptions (project_id)
	where project_id is not null;

create view attribution_enforced_versions as
select v.id
from versions v
where not exists (
	select 1
	from attributions_exemptions ae
	where ae.version_id = v.id or ae.project_id = v.mod_id
);

-- grandfathering migration:
-- insert into attributions_exemptions (version_id)
-- select id
-- from versions
-- on conflict do nothing;
--
-- insert into attributions_exemptions (project_id)
-- select id
-- from mods
-- on conflict do nothing;
