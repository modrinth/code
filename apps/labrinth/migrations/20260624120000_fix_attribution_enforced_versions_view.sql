-- The original view used a single correlated `not exists` with an `or`, which
-- prevents the partial unique indexes on `attributions_exemptions (version_id)`
-- and `(project_id)` from being used, forcing a full sequential scan of the
-- exemptions table for every candidate version.
--
-- `not exists (a or b)` is equivalent to `not exists (a) and not exists (b)`,
-- so splitting the subquery lets each anti-join use its own index.
create or replace view attribution_enforced_versions as
select v.id
from versions v
where not exists (
	select 1
	from attributions_exemptions ae
	where ae.version_id = v.id
)
and not exists (
	select 1
	from attributions_exemptions ae
	where ae.project_id = v.mod_id
);
