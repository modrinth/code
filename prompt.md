read the last commit. basically what we're doing, is we want to store a project's `components` into the search indexing. and when we fetch a project, we fetch its components from the db (the `components` jsonb column), AND each component can fetch some extra stuff for itself. for example, the `minecraft_java_server` project component has a content field which has a modpack id `version_id`. when we query this (either for returning as an API request, or for search indexing), we want to also fetch extra data associated with that version id. BUT, 2 problems:
- we want 1 logical unified place to store the logic for this extra fetch
- BUT in the dbproject::get_many fn (where the fetch is done), the function is fucking massive, and there's no easy way to add a "hook" into there for components
- and I don't want components to have access to the `db: &PgPool` themselves, since then each component does an extra round trip to the db. ideally, we fetch all projects as batch, and any extra data required by components at the same time, then components figure out how to take this extra-fetch data and map it into themselves.
- this same issue is repeated for search - but if we fix the 2 points above, we can reuse this for search

figure out a solution to this. ideas:
- have a fn in `exp.rs` which is essentially `DBProject::get_many`, and it defines a place where all components define what they fetch
- store that extra fetch data as like a `ProjectFetchContext`
- each component type has a `fn map(&ProjectFetchContext, ProjecetId) -> Result<Self>` which takes some of that fetched context, and gets the appropriate data to make an instance of itself
- call this fn from DBProject::get_many and from search indexing ( [@local_import.rs](file:///home/boris/Projects/code3/apps/labrinth/src/search/indexing/local_import.rs) )
