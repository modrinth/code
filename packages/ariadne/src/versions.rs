use std::{collections::HashMap, sync::LazyLock};

static SPECIAL_PARENTS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("15w14a", "1.8.3");
        m.insert("1.RV-Pre1", "1.9.2");
        m.insert("3D Shareware v1.34", "19w13b");
        m.insert("20w14infinite", "20w13b");
        m.insert("22w13oneblockatatime", "1.18.2");
        m.insert("23w13a_or_b", "23w13a");
        m.insert("24w14potato", "24w12a");
        m
    });

pub fn is_feature_supported_in(
    version: &str,
    first_release: &str,
    first_snapshot: &str,
) -> bool {
    let version = SPECIAL_PARENTS.get(version).copied().unwrap_or(version);
    if version.contains('w') && version.len() == 6 {
        return version >= first_snapshot;
    }
    if version == first_release {
        return true;
    }
    let parts_version = version.split('.');
    let parts_first_release = first_release.split('.');
    for (part_version, part_first_release) in
        parts_version.zip(parts_first_release)
    {
        if part_version == part_first_release {
            continue;
        }
        if let Ok(part_version) = part_version.parse::<u32>() {
            if let Ok(part_first_release) = part_first_release.parse::<u32>() {
                if part_version > part_first_release {
                    return true;
                }
            }
        }
    }
    false
}
