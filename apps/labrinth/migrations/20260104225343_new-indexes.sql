-- Adding IF NOT EXISTS because at this time they are already on the production database.
CREATE INDEX IF NOT EXISTS ON public.collections_mods USING btree (mod_id);
CREATE INDEX IF NOT EXISTS ON public.collections USING btree (user_id);
CREATE INDEX IF NOT EXISTS ON public.mods_gallery USING btree (image_url);
CREATE INDEX IF NOT EXISTS ON public.delphi_report_issue_details USING btree (
	issue_id
);
CREATE INDEX IF NOT EXISTS ON public.users USING btree (github_id);
CREATE INDEX IF NOT EXISTS ON public.team_members USING btree (accepted);
CREATE INDEX IF NOT EXISTS ON public.charges USING btree (parent_charge_id);
CREATE INDEX IF NOT EXISTS ON public.charges USING btree (subscription_id);
CREATE INDEX IF NOT EXISTS ON public.charges USING btree (user_id);
CREATE INDEX IF NOT EXISTS ON public.versions USING btree (author_id);
CREATE INDEX IF NOT EXISTS ON public.versions USING btree (status)
WHERE (requested_status IS NOT NULL);
CREATE INDEX IF NOT EXISTS ON public.dependencies USING btree (mod_dependency_id);
CREATE INDEX IF NOT EXISTS ON public.payouts_values USING btree (user_id, created);
CREATE INDEX IF NOT EXISTS ON public.payouts_values USING btree (
	user_id,
	date_available
);
CREATE INDEX IF NOT EXISTS ON public.threads_messages USING btree (author_id);
CREATE INDEX IF NOT EXISTS ON public.threads_messages USING btree (thread_id);
CREATE INDEX IF NOT EXISTS ON public.mod_follows USING btree (mod_id);
