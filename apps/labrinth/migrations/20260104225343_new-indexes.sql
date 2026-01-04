-- Adding IF NOT EXISTS because at this time they are already on the production database.
CREATE INDEX CONCURRENTLY IF NOT EXISTS ON public.collections_mods USING btree (mod_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS ON public.collections USING btree (user_id);
