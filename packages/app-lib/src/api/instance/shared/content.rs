use super::client::{
    ExternalFileResponse, InstanceVersionResponse, get_remote_version,
};
use super::*;
use std::collections::BTreeMap;

const CONFIG_VERSION_FETCH_CONCURRENCY: usize = 4;

/// Config files are published as deltas, so a member needs every config
/// uploaded after `applied_version`, keeping the newest upload of each path.
pub(super) async fn remote_shared_config_files(
    shared_instance_id: &str,
    applied_version: Option<i32>,
    latest: &InstanceVersionResponse,
    state: &State,
) -> crate::Result<Vec<ExternalFileResponse>> {
    use futures::{StreamExt, TryStreamExt};

    let first = applied_version.map_or(1, |version| version.saturating_add(1));
    let intermediate = futures::stream::iter(first..latest.version)
        .map(|version| get_remote_version(shared_instance_id, version, state))
        .buffered(CONFIG_VERSION_FETCH_CONCURRENCY)
        .try_collect::<Vec<_>>()
        .await?;

    newest_config_files(
        intermediate
            .iter()
            .flatten()
            .chain((first <= latest.version).then_some(latest))
            .map(|version| version.external_files.as_slice()),
    )
}

/// Takes each version's external files in ascending version order. A legacy
/// config bundle is a full snapshot, so it replaces every older config. The
/// bundle comes first so newer individual files are written over it.
///
/// Limits apply per version, matching what a single publish may upload.
fn newest_config_files<'a>(
    versions: impl IntoIterator<Item = &'a [ExternalFileResponse]>,
) -> crate::Result<Vec<ExternalFileResponse>> {
    let mut bundle = None;
    let mut files = BTreeMap::new();
    for external_files in versions {
        ensure_config_limits(external_files)?;
        if let Some(newer) = external_files
            .iter()
            .find(|file| file.file_type == CONFIG_BUNDLE_FILE_TYPE)
        {
            bundle = Some(newer.clone());
            files.clear();
        }
        for file in external_files
            .iter()
            .filter(|file| file.file_type == CONFIG_FILE_TYPE)
        {
            files.insert(file.file_name.clone(), file.clone());
        }
    }

    Ok(bundle.into_iter().chain(files.into_values()).collect())
}

fn ensure_config_limits(
    external_files: &[ExternalFileResponse],
) -> crate::Result<()> {
    let mut count = 0;
    let mut size = 0_u64;
    for file in external_files
        .iter()
        .filter(|file| file.file_type == CONFIG_FILE_TYPE)
    {
        count += 1;
        size = size.saturating_add(
            file.file_size
                .and_then(|size| u64::try_from(size).ok())
                .unwrap_or(u64::MAX),
        );
    }
    if count > MAX_SHARED_FILES || size > MAX_SHARED_FILES_TOTAL_SIZE {
        bail!(crate::ErrorKind::InputError(
            "shared instance files exceed the size or file count limit"
                .to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(
        file_type: &str,
        file_name: &str,
        url: &str,
    ) -> ExternalFileResponse {
        ExternalFileResponse {
            file_name: file_name.to_string(),
            file_type: file_type.to_string(),
            url: url.to_string(),
            file_size: Some(1),
            sha1: None,
        }
    }

    fn names(files: &[ExternalFileResponse]) -> Vec<(&str, &str, &str)> {
        files
            .iter()
            .map(|file| {
                (
                    file.file_type.as_str(),
                    file.file_name.as_str(),
                    file.url.as_str(),
                )
            })
            .collect()
    }

    #[test]
    fn newest_upload_of_each_path_wins() {
        let v1 = [
            file(CONFIG_FILE_TYPE, "config/a.toml", "v1"),
            file(CONFIG_FILE_TYPE, "config/b.toml", "v1"),
        ];
        let v2 = [file("mod", "extra.jar", "v2")];
        let v3 = [file(CONFIG_FILE_TYPE, "config/a.toml", "v3")];

        let files = newest_config_files([&v1[..], &v2[..], &v3[..]]).unwrap();

        assert_eq!(
            names(&files),
            [
                (CONFIG_FILE_TYPE, "config/a.toml", "v3"),
                (CONFIG_FILE_TYPE, "config/b.toml", "v1"),
            ]
        );
    }

    #[test]
    fn bundle_replaces_older_configs_and_is_written_first() {
        let v1 = [file(CONFIG_FILE_TYPE, "config/old.toml", "v1")];
        let v2 = [
            file(CONFIG_FILE_TYPE, "config/same.toml", "v2"),
            file(CONFIG_BUNDLE_FILE_TYPE, "configs.zip", "v2"),
        ];
        let v3 = [file(CONFIG_FILE_TYPE, "config/new.toml", "v3")];

        let files = newest_config_files([&v1[..], &v2[..], &v3[..]]).unwrap();

        assert_eq!(
            names(&files),
            [
                (CONFIG_BUNDLE_FILE_TYPE, "configs.zip", "v2"),
                (CONFIG_FILE_TYPE, "config/new.toml", "v3"),
                (CONFIG_FILE_TYPE, "config/same.toml", "v2"),
            ]
        );
    }

    #[test]
    fn limits_apply_per_version() {
        let mut half = file(CONFIG_FILE_TYPE, "config/a.toml", "v1");
        half.file_size = Some((MAX_SHARED_FILES_TOTAL_SIZE / 2 + 1) as i64);
        let mut other_half = half.clone();
        other_half.file_name = "config/b.toml".to_string();
        let mut too_large = half.clone();
        too_large.file_name = "config/c.toml".to_string();

        assert!(
            newest_config_files([
                &[half.clone()][..],
                &[other_half.clone()][..]
            ])
            .is_ok()
        );
        assert!(
            newest_config_files([&[half, other_half, too_large][..]]).is_err()
        );
    }

    #[test]
    fn newest_bundle_wins() {
        let v1 = [file(CONFIG_BUNDLE_FILE_TYPE, "configs.zip", "v1")];
        let v2 = [file(CONFIG_BUNDLE_FILE_TYPE, "configs.zip", "v2")];

        let files = newest_config_files([&v1[..], &v2[..]]).unwrap();

        assert_eq!(
            names(&files),
            [(CONFIG_BUNDLE_FILE_TYPE, "configs.zip", "v2")]
        );
    }
}
