table! {
    mods (id) {
        id -> Int4,
        title -> Varchar,
        description -> Varchar,
        datepublished -> Date,
        author -> Varchar,
        downloads -> Int4,
        categories -> Array<Text>,
        body_path -> Varchar,
        icon_path -> Varchar,
    }
}

table! {
    versions (version_id) {
        version_id -> Int4,
        mod_id -> Int4,
        title -> Varchar,
        changelog_path -> Varchar,
        files_path -> Array<Text>,
        date_published -> Date,
        author -> Varchar,
        downloads -> Int4,
        dependencies -> Array<Text>,
        game_versions -> Array<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    mods,
    versions,
);
