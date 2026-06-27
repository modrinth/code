alter table project_attribution_files
	add constraint project_attribution_files_group_id_sha1_key unique (group_id, sha1);

-- manually run:
--
-- delete from project_attribution_files paf
-- using (
--     select ctid
--     from (
--         select
--             ctid,
--             row_number() over (
--                 partition by group_id, sha1
--                 order by moderation_external_license_id nulls last, name
--             ) as row_number
--         from project_attribution_files
--     ) duplicates
--     where row_number > 1
-- ) duplicates
-- where paf.ctid = duplicates.ctid;
