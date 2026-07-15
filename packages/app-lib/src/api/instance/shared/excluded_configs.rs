pub(crate) const EXCLUDED_CONFIG_FOLDERS: &[&str] = &[
    // REASON: Stores all the mod update information in 1000s of JSON files.
    "notenoughupdates/repo",
    // REASON: 1000s of item JSON defintiions, generated on first launch
    "skyblocker/item-repo",
];
