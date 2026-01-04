-- Adding IF NOT EXISTS because at this time they are already on the production database.
CREATE INDEX CONCURRENTLY IF NOT EXISTS ON public.collections_mods USING btree (mod_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS ON public.collections USING btree (user_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS ON public.delphi_report_issue_details USING btree (
	issue_id
);
