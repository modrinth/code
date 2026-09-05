use modrinth_content_management::{
    Change, CommonExternalFilePolicy, ConfigurationDiff,
    ContentSetConfiguration, ContentSetDiffEntry, ContentSetDiffKind,
    ContentSetDiffOptions, ContentSetSnapshot, ContentType, Error,
    ExternalFileKey, LoaderReference, diff_configuration, diff_content_sets,
};

fn snapshot(
    projects: &[(&str, &str)],
    files: &[(ContentType, &str)],
) -> ContentSetSnapshot {
    ContentSetSnapshot {
        projects: projects
            .iter()
            .map(|(project, version)| {
                (project.to_string(), version.to_string())
            })
            .collect(),
        external_files: files
            .iter()
            .map(|(content_type, path)| file(*content_type, path))
            .collect(),
    }
}

fn file(content_type: ContentType, path: &str) -> ExternalFileKey {
    ExternalFileKey {
        content_type,
        path: path.to_string(),
    }
}

fn configuration(
    modpack_version_id: Option<&str>,
    game_version: &str,
    loader: &str,
    loader_version: Option<&str>,
) -> ContentSetConfiguration {
    ContentSetConfiguration {
        modpack_version_id: modpack_version_id.map(str::to_string),
        game_version: game_version.to_string(),
        loader: LoaderReference {
            name: loader.to_string(),
            version: loader_version.map(str::to_string),
        },
    }
}

#[test]
fn empty_and_identical_snapshots_have_no_changes() {
    for content in [
        ContentSetSnapshot::default(),
        snapshot(&[("sodium", "v1")], &[(ContentType::Mod, "custom.jar")]),
    ] {
        let diff = diff_content_sets(
            &content,
            &content,
            &ContentSetDiffOptions::default(),
        );
        assert!(!diff.has_changes());
        assert!(diff.content.is_empty());
        assert!(diff.additional.is_empty());
    }
}

#[test]
fn project_changes_preserve_identity_and_both_version_ids() {
    let before = snapshot(
        &[("unchanged", "v1"), ("updated", "old"), ("removed", "v2")],
        &[],
    );
    let after = snapshot(
        &[("updated", "new"), ("added", "v3"), ("unchanged", "v1")],
        &[],
    );

    let diff =
        diff_content_sets(&before, &after, &ContentSetDiffOptions::default());

    assert!(diff.has_changes());
    assert_eq!(
        diff.content,
        vec![
            ContentSetDiffEntry::Project {
                project_id: "added".to_string(),
                change: Change::Added {
                    after: "v3".to_string()
                },
            },
            ContentSetDiffEntry::Project {
                project_id: "removed".to_string(),
                change: Change::Removed {
                    before: "v2".to_string()
                },
            },
            ContentSetDiffEntry::Project {
                project_id: "updated".to_string(),
                change: Change::Updated {
                    before: "old".to_string(),
                    after: "new".to_string(),
                },
            },
        ]
    );
}

#[test]
fn reversing_snapshots_reverses_project_and_file_changes() {
    let before = snapshot(
        &[("updated", "old"), ("removed", "v1")],
        &[(ContentType::Mod, "removed.jar")],
    );
    let after = snapshot(
        &[("updated", "new"), ("added", "v2")],
        &[(ContentType::Mod, "added.jar")],
    );

    let reverse =
        diff_content_sets(&after, &before, &ContentSetDiffOptions::default());

    assert_eq!(
        reverse.content,
        vec![
            ContentSetDiffEntry::Project {
                project_id: "added".to_string(),
                change: Change::Removed {
                    before: "v2".to_string()
                },
            },
            ContentSetDiffEntry::Project {
                project_id: "removed".to_string(),
                change: Change::Added {
                    after: "v1".to_string()
                },
            },
            ContentSetDiffEntry::Project {
                project_id: "updated".to_string(),
                change: Change::Updated {
                    before: "new".to_string(),
                    after: "old".to_string(),
                },
            },
            ContentSetDiffEntry::ExternalFile {
                file: file(ContentType::Mod, "added.jar"),
                kind: ContentSetDiffKind::Removed,
            },
            ContentSetDiffEntry::ExternalFile {
                file: file(ContentType::Mod, "removed.jar"),
                kind: ContentSetDiffKind::Added,
            },
        ]
    );
}

#[test]
fn common_file_policy_only_affects_files_present_on_both_sides() {
    let before = snapshot(
        &[],
        &[
            (ContentType::Mod, "common.jar"),
            (ContentType::Mod, "removed.jar"),
        ],
    );
    let after = snapshot(
        &[],
        &[
            (ContentType::Mod, "added.jar"),
            (ContentType::Mod, "common.jar"),
        ],
    );
    let added = ContentSetDiffEntry::ExternalFile {
        file: file(ContentType::Mod, "added.jar"),
        kind: ContentSetDiffKind::Added,
    };
    let removed = ContentSetDiffEntry::ExternalFile {
        file: file(ContentType::Mod, "removed.jar"),
        kind: ContentSetDiffKind::Removed,
    };
    let common = ContentSetDiffEntry::ExternalFile {
        file: file(ContentType::Mod, "common.jar"),
        kind: ContentSetDiffKind::Updated,
    };
    for (policy, expected) in [
        (
            CommonExternalFilePolicy::AssumeUnchanged,
            vec![added.clone(), removed.clone()],
        ),
        (
            CommonExternalFilePolicy::AssumeUpdated,
            vec![added, common, removed],
        ),
    ] {
        let diff = diff_content_sets(
            &before,
            &after,
            &ContentSetDiffOptions {
                common_external_files: policy,
            },
        );
        assert_eq!(diff.content, expected, "{policy:?}");
    }
}

#[test]
fn assumed_common_file_update_counts_as_a_change() {
    let content = snapshot(&[], &[(ContentType::ResourcePack, "textures.zip")]);
    let diff = diff_content_sets(
        &content,
        &content,
        &ContentSetDiffOptions {
            common_external_files: CommonExternalFilePolicy::AssumeUpdated,
        },
    );

    assert!(diff.has_changes());
    assert_eq!(
        diff.content,
        vec![ContentSetDiffEntry::ExternalFile {
            file: file(ContentType::ResourcePack, "textures.zip"),
            kind: ContentSetDiffKind::Updated,
        }]
    );
}

#[test]
fn file_identity_includes_content_type_and_relative_path() {
    let before = snapshot(
        &[],
        &[
            (ContentType::Mod, "shared.jar"),
            (ContentType::DataPack, "first/data.zip"),
        ],
    );
    let after = snapshot(
        &[],
        &[
            (ContentType::Plugin, "shared.jar"),
            (ContentType::DataPack, "second/data.zip"),
        ],
    );

    let diff =
        diff_content_sets(&before, &after, &ContentSetDiffOptions::default());

    assert_eq!(
        diff.content,
        vec![
            ContentSetDiffEntry::ExternalFile {
                file: file(ContentType::Mod, "shared.jar"),
                kind: ContentSetDiffKind::Removed,
            },
            ContentSetDiffEntry::ExternalFile {
                file: file(ContentType::Plugin, "shared.jar"),
                kind: ContentSetDiffKind::Added,
            },
            ContentSetDiffEntry::ExternalFile {
                file: file(ContentType::DataPack, "first/data.zip"),
                kind: ContentSetDiffKind::Removed,
            },
            ContentSetDiffEntry::ExternalFile {
                file: file(ContentType::DataPack, "second/data.zip"),
                kind: ContentSetDiffKind::Added,
            },
        ]
    );
}

#[test]
fn diff_order_is_independent_of_snapshot_insertion_order() {
    let first = snapshot(
        &[("z", "v2"), ("a", "v1")],
        &[(ContentType::Mod, "z.jar"), (ContentType::Mod, "a.jar")],
    );
    let second = snapshot(
        &[("a", "v1"), ("z", "v2")],
        &[(ContentType::Mod, "a.jar"), (ContentType::Mod, "z.jar")],
    );
    let empty = ContentSetSnapshot::default();
    let options = ContentSetDiffOptions::default();

    assert_eq!(
        diff_content_sets(&empty, &first, &options),
        diff_content_sets(&empty, &second, &options),
    );
}

#[test]
fn inserting_the_same_project_version_is_idempotent() {
    let mut content = ContentSetSnapshot::default();
    content
        .insert_project("sodium".to_string(), "v1".to_string())
        .unwrap();
    let original = content.clone();

    content
        .insert_project("sodium".to_string(), "v1".to_string())
        .unwrap();

    assert_eq!(content, original);
}

#[test]
fn conflicting_project_version_is_rejected_without_changing_the_snapshot() {
    let mut content = snapshot(&[("sodium", "v1")], &[]);
    let original = content.clone();

    let error = content
        .insert_project("sodium".to_string(), "v2".to_string())
        .unwrap_err();

    assert!(
        matches!(error, Error::ConflictingProjectVersions { project_id, before, after }
		if project_id == "sodium" && before == "v1" && after == "v2")
    );
    assert_eq!(content, original);
}

#[derive(Debug, PartialEq)]
enum AdditionalChange {
    ConfigFilesUpdated { file_count: usize },
}

#[test]
fn caller_entries_can_trigger_changes_without_content_changes() {
    let content = ContentSetSnapshot::default();
    let base = diff_content_sets(
        &content,
        &content,
        &ContentSetDiffOptions::default(),
    );
    let mut diff = base.with_additional(Vec::<AdditionalChange>::new());
    assert!(!diff.has_changes());

    diff.additional
        .push(AdditionalChange::ConfigFilesUpdated { file_count: 2 });

    assert!(diff.content.is_empty());
    assert!(diff.has_changes());
    assert_eq!(
        diff.additional,
        vec![AdditionalChange::ConfigFilesUpdated { file_count: 2 }]
    );
}

#[test]
fn attaching_caller_entries_preserves_content_changes() {
    let before = ContentSetSnapshot::default();
    let after = snapshot(&[("sodium", "v1")], &[]);
    let base =
        diff_content_sets(&before, &after, &ContentSetDiffOptions::default());
    let content = base.content.clone();
    let mut diff =
        base.with_additional([AdditionalChange::ConfigFilesUpdated {
            file_count: 1,
        }]);

    assert_eq!(diff.content, content);
    assert_eq!(diff.additional.len(), 1);
    diff.additional.clear();
    assert!(diff.has_changes());
}

#[test]
fn modpack_link_unlink_and_update_have_directional_version_ids() {
    for (before_id, after_id, expected) in [
        (
            None,
            Some("pack-v1"),
            Change::Added {
                after: "pack-v1".to_string(),
            },
        ),
        (
            Some("pack-v1"),
            None,
            Change::Removed {
                before: "pack-v1".to_string(),
            },
        ),
        (
            Some("pack-v1"),
            Some("pack-v2"),
            Change::Updated {
                before: "pack-v1".to_string(),
                after: "pack-v2".to_string(),
            },
        ),
    ] {
        let before =
            configuration(before_id, "1.21.1", "fabric", Some("0.16.0"));
        let after = configuration(after_id, "1.21.1", "fabric", Some("0.16.0"));

        assert_eq!(
            diff_configuration(&before, &after),
            vec![ConfigurationDiff::Modpack(expected)]
        );
    }
}

#[test]
fn configuration_normalization_does_not_create_false_changes() {
    let empty = configuration(None, "1.21.1", "fabric", None);
    let explicit_empty = configuration(Some(""), "1.21.1", "Fabric", Some(""));
    let linked =
        configuration(Some("pack-v1"), "1.21.1", "NeoForge", Some("21.1.1"));
    let canonical =
        configuration(Some("pack-v1"), "1.21.1", "neoforge", Some("21.1.1"));

    for (before, after) in [(&empty, &explicit_empty), (&linked, &canonical)] {
        assert!(diff_configuration(before, after).is_empty());
        assert!(diff_configuration(after, before).is_empty());
        assert!(diff_configuration(before, before).is_empty());
    }
}

#[test]
fn changing_only_the_game_version_produces_one_configuration_entry() {
    let before = configuration(None, "1.21.1", "fabric", Some("0.16.0"));
    let after = configuration(None, "1.21.2", "fabric", Some("0.16.0"));

    assert_eq!(
        diff_configuration(&before, &after),
        vec![ConfigurationDiff::GameVersion(Change::Updated {
            before: "1.21.1".to_string(),
            after: "1.21.2".to_string(),
        }),]
    );
}

#[test]
fn loader_version_changes_include_added_and_removed_optional_versions() {
    for (before_version, after_version) in [
        (Some("0.16.0"), Some("0.16.1")),
        (None, Some("0.16.0")),
        (Some("0.16.0"), None),
    ] {
        let before = configuration(None, "1.21.1", "fabric", before_version);
        let after = configuration(None, "1.21.1", "fabric", after_version);

        assert_eq!(
            diff_configuration(&before, &after),
            vec![ConfigurationDiff::Loader(Change::Updated {
                before: before.loader.clone(),
                after: after.loader.clone(),
            }),]
        );
    }
}

#[test]
fn compatible_loader_names_still_represent_different_installed_loaders() {
    let before = configuration(None, "1.21.1", "paper", Some("123"));
    let after = configuration(None, "1.21.1", "purpur", Some("123"));

    assert_eq!(
        diff_configuration(&before, &after),
        vec![ConfigurationDiff::Loader(Change::Updated {
            before: before.loader.clone(),
            after: after.loader.clone(),
        }),]
    );
}

#[test]
fn simultaneous_configuration_changes_are_composed_in_stable_order() {
    let before =
        configuration(Some("pack-v1"), "1.21.1", "fabric", Some("0.16.0"));
    let after =
        configuration(Some("pack-v2"), "1.21.2", "neoforge", Some("21.2.1"));
    let content = ContentSetSnapshot::default();
    let diff = diff_content_sets(
        &content,
        &content,
        &ContentSetDiffOptions::default(),
    )
    .with_additional(diff_configuration(&before, &after));

    assert!(diff.content.is_empty());
    assert!(diff.has_changes());
    assert_eq!(
        diff.additional,
        vec![
            ConfigurationDiff::Modpack(Change::Updated {
                before: "pack-v1".to_string(),
                after: "pack-v2".to_string(),
            }),
            ConfigurationDiff::GameVersion(Change::Updated {
                before: "1.21.1".to_string(),
                after: "1.21.2".to_string(),
            }),
            ConfigurationDiff::Loader(Change::Updated {
                before: before.loader,
                after: after.loader,
            }),
        ]
    );
}

#[test]
fn mapping_changes_preserves_their_kind_and_before_after_values() {
    for (change, kind, before, after) in [
        (
            Change::Added { after: 2 },
            ContentSetDiffKind::Added,
            None,
            Some("v2"),
        ),
        (
            Change::Removed { before: 1 },
            ContentSetDiffKind::Removed,
            Some("v1"),
            None,
        ),
        (
            Change::Updated {
                before: 1,
                after: 2,
            },
            ContentSetDiffKind::Updated,
            Some("v1"),
            Some("v2"),
        ),
    ] {
        let mapped = change.map(|version| format!("v{version}"));
        assert_eq!(mapped.kind(), kind);
        assert_eq!(mapped.before().map(String::as_str), before);
        assert_eq!(mapped.after().map(String::as_str), after);
    }
}
