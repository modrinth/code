use bytes::Bytes;
use std::io::Cursor;
use zip::ZipArchive;

const OVERRIDE_PREFIXES: &[&str] = &[
    "overrides/mods",
    "client-overrides/mods",
    "server-overrides/mods",
    "overrides/shaderpacks",
    "client-overrides/shaderpacks",
    "overrides/resourcepacks",
    "client-overrides/resourcepacks",
];

pub fn check_override_licenses(
    data: &Bytes,
) -> Result<(), zip::result::ZipError> {
    let reader = Cursor::new(data);
    let mut zip = ZipArchive::new(reader)?;

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;

        let name = file.name();

        if !OVERRIDE_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(prefix))
        {
            continue;
        }

        if name.matches('/').count() > 2
            || name.ends_with(".txt")
            || name.ends_with(".rpo")
        {
            continue;
        }

        println!("{}", name);
    }

    Ok(())
}
