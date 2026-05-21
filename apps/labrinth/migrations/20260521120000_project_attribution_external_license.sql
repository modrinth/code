alter table project_attribution_files
	add column moderation_external_license_id bigint references moderation_external_licenses(id);

create table version_attribution_exemptions (
	version_id bigint primary key references versions(id) on delete cascade
);

create view attribution_enforced_versions as
select v.id
from versions v
left join version_attribution_exemptions vae on vae.version_id = v.id
where vae.version_id is null;

-- grandfathering migration:
-- insert into version_attribution_exemptions (version_id)
-- select v.id
-- from versions v
-- inner join mods m on m.id = v.mod_id
-- where m.status in ('approved', 'unlisted', 'archived', 'private', 'scheduled', 'withheld')
-- on conflict do nothing;
