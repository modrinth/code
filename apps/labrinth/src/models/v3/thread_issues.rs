use crate::models::disclosures::ProjectDisclosure;
use crate::models::exp::minecraft::Language;
use crate::models::ids::{
    FileId, GalleryImageId, TeamId, ThreadIssueId, VersionId,
};
use crate::models::projects::{Dependency, FileType, Project, Version};
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current project state used to evaluate moderation issues.
#[derive(Debug, Clone, Copy)]
pub struct ThreadIssueContext<'a> {
    pub project: &'a Project,
    pub versions: &'a [Version],
    pub disclosures: &'a [ProjectDisclosure],
    pub team_members: &'a [ThreadIssueTeamMember],
}

#[derive(Debug, Clone)]
pub struct ThreadIssueTeamMember {
    pub team_id: TeamId,
    pub user_id: UserId,
    pub role: String,
}

/// Issue that a moderator has flagged on a project in its moderation thread.
///
/// If a moderator has a specific, targeted, actionable piece of feedback on a
/// specific part of a project (e.g. the description, summary, a gallery image,
/// etc.), they can add an issue to the moderation thread which targets that
/// part specifically, along with one of an enumerated set of reasons for why
/// the issue was applied.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ThreadIssue {
    pub id: ThreadIssueId,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    /// What part of a project this issue applies to.
    pub what: ThreadIssueTarget,
    /// Why the issue was raised on this part.
    ///
    /// This is treated as an opaque JSON blob by the backend; it is up to the
    /// frontend to define a schema for it, and to render/localize it properly.
    pub why: serde_json::Value,
    /// Has the user confirmed that they've seen this issue and (attempted to)
    /// resolve it?
    pub user_addressed: bool,
    /// Has a moderator seen the user's resolution and explicitly marked it as
    /// resolved?
    ///
    /// If a moderator marks this issue as verified, then it will locked to
    /// [`ThreadIssueVerdict::Resolved`].
    pub moderator_verified: bool,
    /// Final derived verdict of this issue.
    pub verdict: ThreadIssueVerdict,
}

/// What part of a project must change for a [`ThreadIssue`] to be resolved?
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ThreadIssueTarget {
    ModifyTitle(TextTarget),
    ModifySlug(TextTarget),
    ModifySummary(TextTarget),
    ModifyDescription(TextTarget),
    ModifyLicense {
        license: TextTarget,
        url: TextTarget,
    },
    ModifyIcon {
        original_url: Option<String>,
    },
    RemoveTags {
        tags: Vec<String>,
    },
    ModifyLinks {
        links: HashMap<String, TextTarget>,
    },
    AddGalleryImages {
        original_count: u32,
    },
    ModifyGalleryImage {
        image_id: GalleryImageId,
        original_url: String,
        name: Option<OptionalTextTarget>,
        description: Option<OptionalTextTarget>,
    },
    RemoveGalleryImages {
        image_ids: Vec<GalleryImageId>,
    },
    RemoveProjectDisclosures {
        disclosure_types: Vec<String>,
    },
    ModifyProjectDisclosure {
        disclosure_type: String,
        metadata: JsonTarget,
    },
    ModifyProjectDisclosureNote {
        disclosure_type: String,
        note: OptionalTextTarget,
    },
    Version {
        version_id: VersionId,
        version_number: String,
        target: VersionIssueTarget,
    },
    ModifyTeamMemberRole {
        team_id: TeamId,
        user_id: UserId,
        role: TextTarget,
    },
    ModifyServerLanguages {
        original: Vec<Language>,
        suggestion: Option<Vec<Language>>,
    },
    ModifyServerAddress {
        platform: ServerAddressPlatform,
        address: TextTarget,
    },
    Acknowledge {
        mode: ThreadIssueAcknowledgement,
    },
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
pub struct TextTarget {
    /// Original value of this field when the [`ThreadIssue`] was made.
    pub original: String,
    /// Moderator-proposed value for this field.
    pub suggestion: Option<String>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
pub struct OptionalTextTarget {
    pub original: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub suggestion: Option<Option<String>>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
pub struct JsonTarget {
    pub original: serde_json::Value,
    pub suggestion: Option<serde_json::Value>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum VersionIssueTarget {
    Remove,
    ModifyEnvironment(TextTarget),
    ModifyGameVersions {
        original: Vec<String>,
        suggestion: Option<Vec<String>>,
    },
    ModifyDependencies {
        original: Vec<Dependency>,
        suggestion: Option<Vec<Dependency>>,
    },
    ModifyChangelog(TextTarget),
    RemoveAdditionalFiles {
        file_ids: Vec<FileId>,
    },
    ModifyAdditionalFileType {
        file_id: FileId,
        filename: String,
        original: Option<FileType>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "::serde_with::rust::double_option"
        )]
        suggestion: Option<Option<FileType>>,
    },
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ServerAddressPlatform {
    MinecraftJava,
    MinecraftBedrock,
}

/// How should a user communicate that they've resolved a [`ThreadIssue`]?
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ThreadIssueAcknowledgement {
    /// They must tick a checkbox.
    Checkbox,
    /// They must reply to the thread with an explanation message.
    Reply,
}

/// Current state of the action required to resolve a [`ThreadIssue`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ThreadIssueValueState {
    /// Current field value is the same as it was originally.
    ///
    /// It may have never been changed, or been changed multiple times, ending
    /// up at the original value again.
    SameAsOriginal,
    /// Current field value is different to the original value.
    DifferentToOriginal,
    /// Current field value is equal to the moderator's proposed value.
    SameAsSuggested,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ThreadIssueVerdict {
    Open,
    Addressed,
    Resolved,
}

impl ThreadIssueVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Addressed => "addressed",
            Self::Resolved => "resolved",
        }
    }
}

impl ThreadIssueTarget {
    pub fn verdict(
        &self,
        context: &ThreadIssueContext<'_>,
        user_addressed: bool,
        moderator_verified: bool,
    ) -> ThreadIssueVerdict {
        if moderator_verified {
            return ThreadIssueVerdict::Resolved;
        }

        match (self.value_state(context, user_addressed), user_addressed) {
            (ThreadIssueValueState::SameAsSuggested, _) => {
                ThreadIssueVerdict::Resolved
            }
            (ThreadIssueValueState::DifferentToOriginal, true) => {
                ThreadIssueVerdict::Addressed
            }
            _ => ThreadIssueVerdict::Open,
        }
    }

    pub fn value_state(
        &self,
        context: &ThreadIssueContext<'_>,
        user_addressed: bool,
    ) -> ThreadIssueValueState {
        let project = context.project;
        match self {
            Self::ModifyTitle(target) => value_state(target, &project.name),
            Self::ModifySlug(target) => {
                value_state(target, project.slug.as_deref().unwrap_or_default())
            }
            Self::ModifySummary(target) => {
                value_state(target, &project.summary)
            }
            Self::ModifyDescription(target) => {
                value_state(target, &project.description)
            }
            Self::ModifyLicense { license, url } => {
                let license_state =
                    value_state(license, project.license.id.as_str());
                let url_state = value_state(
                    url,
                    project.license.url.as_deref().unwrap_or_default(),
                );

                match (license_state, url_state) {
                    (ThreadIssueValueState::DifferentToOriginal, _)
                    | (_, ThreadIssueValueState::DifferentToOriginal) => {
                        ThreadIssueValueState::DifferentToOriginal
                    }
                    (ThreadIssueValueState::SameAsSuggested, _)
                    | (_, ThreadIssueValueState::SameAsSuggested) => {
                        ThreadIssueValueState::SameAsSuggested
                    }
                    _ => ThreadIssueValueState::SameAsOriginal,
                }
            }
            Self::ModifyIcon { original_url } => {
                if project.icon_url == *original_url {
                    ThreadIssueValueState::SameAsOriginal
                } else {
                    ThreadIssueValueState::DifferentToOriginal
                }
            }
            Self::RemoveTags { tags } => {
                let remaining = tags
                    .iter()
                    .filter(|tag| {
                        project.categories.contains(tag)
                            || project.additional_categories.contains(tag)
                    })
                    .count();
                if remaining == 0 {
                    ThreadIssueValueState::SameAsSuggested
                } else if remaining == tags.len() {
                    ThreadIssueValueState::SameAsOriginal
                } else {
                    ThreadIssueValueState::DifferentToOriginal
                }
            }
            Self::ModifyLinks { links } => {
                let mut state = ThreadIssueValueState::SameAsOriginal;
                for (platform, target) in links {
                    let current = project
                        .link_urls
                        .get(platform)
                        .map(|link| link.url.as_str())
                        .unwrap_or_default();
                    match value_state(target, current) {
                        ThreadIssueValueState::DifferentToOriginal => {
                            state = ThreadIssueValueState::DifferentToOriginal;
                            break;
                        }
                        ThreadIssueValueState::SameAsSuggested => {
                            state = ThreadIssueValueState::SameAsSuggested;
                        }
                        ThreadIssueValueState::SameAsOriginal => {}
                    }
                }
                state
            }
            Self::AddGalleryImages { original_count } => {
                if project.gallery.len() > *original_count as usize {
                    ThreadIssueValueState::DifferentToOriginal
                } else {
                    ThreadIssueValueState::SameAsOriginal
                }
            }
            Self::ModifyGalleryImage {
                image_id,
                name,
                description,
                ..
            } => {
                let Some(image) = project
                    .gallery
                    .iter()
                    .find(|image| image.id.as_ref() == Some(image_id))
                else {
                    return ThreadIssueValueState::DifferentToOriginal;
                };

                let name_state = name.as_ref().map(|target| {
                    optional_text_value_state(target, &image.name)
                });
                let description_state = description.as_ref().map(|target| {
                    optional_text_value_state(target, &image.description)
                });

                if name_state
                    .into_iter()
                    .chain(description_state)
                    .all(|state| state == ThreadIssueValueState::SameAsOriginal)
                {
                    ThreadIssueValueState::SameAsOriginal
                } else if name_state.into_iter().chain(description_state).all(
                    |state| state == ThreadIssueValueState::SameAsSuggested,
                ) {
                    ThreadIssueValueState::SameAsSuggested
                } else {
                    ThreadIssueValueState::DifferentToOriginal
                }
            }
            Self::RemoveGalleryImages { image_ids } => {
                let remaining = image_ids
                    .iter()
                    .filter(|id| {
                        project
                            .gallery
                            .iter()
                            .any(|image| image.id.as_ref() == Some(id))
                    })
                    .count();

                if remaining == 0 {
                    ThreadIssueValueState::SameAsSuggested
                } else if remaining == image_ids.len() {
                    ThreadIssueValueState::SameAsOriginal
                } else {
                    ThreadIssueValueState::DifferentToOriginal
                }
            }
            Self::RemoveProjectDisclosures { disclosure_types } => {
                let remaining = disclosure_types
                    .iter()
                    .filter(|target| {
                        context.disclosures.iter().any(|disclosure| {
                            disclosure.to_parts().is_ok_and(
                                |(current_type, _)| current_type == *target,
                            )
                        })
                    })
                    .count();

                if remaining == 0 {
                    ThreadIssueValueState::SameAsSuggested
                } else if remaining == disclosure_types.len() {
                    ThreadIssueValueState::SameAsOriginal
                } else {
                    ThreadIssueValueState::DifferentToOriginal
                }
            }
            Self::ModifyProjectDisclosure {
                disclosure_type,
                metadata,
            } => {
                let current =
                    context.disclosures.iter().find_map(|disclosure| {
                        let (current_type, mut current) =
                            disclosure.to_parts().ok()?;
                        if current_type != disclosure_type {
                            return None;
                        }
                        if let Some(current) = current.as_object_mut() {
                            current.remove("note");
                        }
                        Some(current)
                    });

                let mut original = metadata.original.clone();
                if let Some(original) = original.as_object_mut() {
                    original.remove("note");
                }
                let suggestion =
                    metadata.suggestion.as_ref().map(|suggestion| {
                        let mut suggestion = suggestion.clone();
                        if let Some(suggestion) = suggestion.as_object_mut() {
                            suggestion.remove("note");
                        }
                        suggestion
                    });

                match current {
                    Some(current) if suggestion.as_ref() == Some(&current) => {
                        ThreadIssueValueState::SameAsSuggested
                    }
                    Some(current) if current == original => {
                        ThreadIssueValueState::SameAsOriginal
                    }
                    _ => ThreadIssueValueState::DifferentToOriginal,
                }
            }
            Self::ModifyProjectDisclosureNote {
                disclosure_type,
                note,
            } => {
                let current =
                    context.disclosures.iter().find_map(|disclosure| {
                        let (current_type, metadata) =
                            disclosure.to_parts().ok()?;
                        if current_type != disclosure_type {
                            return None;
                        }
                        Some(
                            metadata
                                .get("note")
                                .and_then(serde_json::Value::as_str)
                                .map(String::from),
                        )
                    });

                current.map_or(
                    ThreadIssueValueState::DifferentToOriginal,
                    |current| optional_text_value_state(note, &current),
                )
            }
            Self::Version {
                version_id, target, ..
            } => {
                let version = context
                    .versions
                    .iter()
                    .find(|version| version.id == *version_id);

                if matches!(target, VersionIssueTarget::Remove) {
                    if version.is_none() {
                        ThreadIssueValueState::SameAsSuggested
                    } else {
                        ThreadIssueValueState::SameAsOriginal
                    }
                } else {
                    let Some(version) = version else {
                        return ThreadIssueValueState::DifferentToOriginal;
                    };

                    match target {
                        VersionIssueTarget::Remove => unreachable!(),
                        VersionIssueTarget::ModifyEnvironment(target) => {
                            value_state(
                                target,
                                version
                                    .fields
                                    .get("environment")
                                    .and_then(serde_json::Value::as_str)
                                    .unwrap_or_default(),
                            )
                        }
                        VersionIssueTarget::ModifyGameVersions {
                            original,
                            suggestion,
                        } => {
                            let current = version
                                .fields
                                .get("game_versions")
                                .and_then(serde_json::Value::as_array)
                                .map(|versions| {
                                    versions
                                        .iter()
                                        .filter_map(|version| {
                                            version.as_str().map(String::from)
                                        })
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default();
                            if suggestion.as_ref().is_some_and(|suggestion| {
                                suggestion.len() == current.len()
                                    && suggestion.iter().all(|version| {
                                        current.contains(version)
                                    })
                            }) {
                                ThreadIssueValueState::SameAsSuggested
                            } else if original.len() == current.len()
                                && original
                                    .iter()
                                    .all(|version| current.contains(version))
                            {
                                ThreadIssueValueState::SameAsOriginal
                            } else {
                                ThreadIssueValueState::DifferentToOriginal
                            }
                        }
                        VersionIssueTarget::ModifyDependencies {
                            original,
                            suggestion,
                        } => {
                            if suggestion.as_ref().is_some_and(|suggestion| {
                                suggestion.len() == version.dependencies.len()
                                    && suggestion.iter().all(|dependency| {
                                        version
                                            .dependencies
                                            .contains(dependency)
                                    })
                            }) {
                                ThreadIssueValueState::SameAsSuggested
                            } else if original.len()
                                == version.dependencies.len()
                                && original.iter().all(|dependency| {
                                    version.dependencies.contains(dependency)
                                })
                            {
                                ThreadIssueValueState::SameAsOriginal
                            } else {
                                ThreadIssueValueState::DifferentToOriginal
                            }
                        }
                        VersionIssueTarget::ModifyChangelog(target) => {
                            value_state(
                                target,
                                version
                                    .changelog
                                    .as_deref()
                                    .unwrap_or_default(),
                            )
                        }
                        VersionIssueTarget::RemoveAdditionalFiles {
                            file_ids,
                        } => {
                            let remaining = file_ids
                                .iter()
                                .filter(|id| {
                                    version.files.iter().any(|file| {
                                        file.id.as_ref() == Some(id)
                                    })
                                })
                                .count();
                            if remaining == 0 {
                                ThreadIssueValueState::SameAsSuggested
                            } else if remaining == file_ids.len() {
                                ThreadIssueValueState::SameAsOriginal
                            } else {
                                ThreadIssueValueState::DifferentToOriginal
                            }
                        }
                        VersionIssueTarget::ModifyAdditionalFileType {
                            file_id,
                            original,
                            suggestion,
                            ..
                        } => {
                            let Some(file) = version
                                .files
                                .iter()
                                .find(|file| file.id.as_ref() == Some(file_id))
                            else {
                                return ThreadIssueValueState::DifferentToOriginal;
                            };
                            if suggestion.as_ref() == Some(&file.file_type) {
                                ThreadIssueValueState::SameAsSuggested
                            } else if file.file_type == *original {
                                ThreadIssueValueState::SameAsOriginal
                            } else {
                                ThreadIssueValueState::DifferentToOriginal
                            }
                        }
                    }
                }
            }
            Self::ModifyTeamMemberRole {
                team_id,
                user_id,
                role,
            } => context
                .team_members
                .iter()
                .find(|member| {
                    member.team_id == *team_id && member.user_id == *user_id
                })
                .map_or(ThreadIssueValueState::DifferentToOriginal, |member| {
                    value_state(role, &member.role)
                }),
            Self::ModifyServerLanguages {
                original,
                suggestion,
            } => {
                let Some(server) = project.components.minecraft_server.as_ref()
                else {
                    return ThreadIssueValueState::DifferentToOriginal;
                };
                if suggestion.as_ref() == Some(&server.languages) {
                    ThreadIssueValueState::SameAsSuggested
                } else if server.languages == *original {
                    ThreadIssueValueState::SameAsOriginal
                } else {
                    ThreadIssueValueState::DifferentToOriginal
                }
            }
            Self::ModifyServerAddress { platform, address } => {
                let current = match platform {
                    ServerAddressPlatform::MinecraftJava => project
                        .components
                        .minecraft_java_server
                        .as_ref()
                        .map(|server| server.address.as_str()),
                    ServerAddressPlatform::MinecraftBedrock => project
                        .components
                        .minecraft_bedrock_server
                        .as_ref()
                        .map(|server| server.address.as_str()),
                };
                current.map_or(
                    ThreadIssueValueState::DifferentToOriginal,
                    |current| value_state(address, current),
                )
            }
            Self::Acknowledge { mode } => match (mode, user_addressed) {
                (_, false) => ThreadIssueValueState::SameAsOriginal,
                (ThreadIssueAcknowledgement::Checkbox, true) => {
                    ThreadIssueValueState::SameAsSuggested
                }
                (ThreadIssueAcknowledgement::Reply, true) => {
                    ThreadIssueValueState::DifferentToOriginal
                }
            },
        }
    }
}

fn optional_text_value_state(
    target: &OptionalTextTarget,
    current: &Option<String>,
) -> ThreadIssueValueState {
    if target.suggestion.as_ref() == Some(current) {
        ThreadIssueValueState::SameAsSuggested
    } else if *current == target.original {
        ThreadIssueValueState::SameAsOriginal
    } else {
        ThreadIssueValueState::DifferentToOriginal
    }
}

fn value_state(target: &TextTarget, current: &str) -> ThreadIssueValueState {
    if target.suggestion.as_deref() == Some(current) {
        ThreadIssueValueState::SameAsSuggested
    } else if current == target.original {
        ThreadIssueValueState::SameAsOriginal
    } else {
        ThreadIssueValueState::DifferentToOriginal
    }
}

impl ThreadIssue {
    pub fn from(data: crate::database::models::DBThreadIssue) -> Self {
        Self {
            id: data.id.into(),
            created_by: data.created_by.into(),
            what: data.what,
            why: data.why,
            user_addressed: data.user_addressed,
            moderator_verified: data.moderator_verified,
            verdict: data.verdict,
            created_at: data.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::exp;
    use crate::models::ids::{ProjectId, TeamId, ThreadId};
    use crate::models::projects::{
        GalleryItem, License, Link, MonetizationStatus, ProjectStatus,
        SideTypesMigrationReviewStatus,
    };

    fn project() -> Project {
        let now = Utc::now();
        Project {
            id: ProjectId(1),
            slug: Some("example-project".to_string()),
            project_types: vec!["mod".to_string()],
            games: vec!["minecraft".to_string()],
            team_id: TeamId(2),
            organization: None,
            name: "Example Project".to_string(),
            summary: "Example summary".to_string(),
            description: "Example description".to_string(),
            published: now,
            updated: now,
            approved: None,
            queued: None,
            status: ProjectStatus::Draft,
            requested_status: None,
            moderator_message: None,
            license: License {
                id: "MIT".to_string(),
                name: "MIT License".to_string(),
                url: Some("https://example.com/license".to_string()),
            },
            downloads: 0,
            followers: 0,
            categories: vec!["technology".to_string()],
            additional_categories: vec!["utility".to_string()],
            loaders: vec!["fabric".to_string()],
            versions: Vec::new(),
            icon_url: Some("https://example.com/icon.png".to_string()),
            raw_icon_url: None,
            link_urls: HashMap::from([
                (
                    "source".to_string(),
                    Link {
                        platform: "source".to_string(),
                        donation: false,
                        url: "https://example.com/source".to_string(),
                    },
                ),
                (
                    "issues".to_string(),
                    Link {
                        platform: "issues".to_string(),
                        donation: false,
                        url: "https://example.com/issues".to_string(),
                    },
                ),
            ]),
            gallery: vec![
                GalleryItem {
                    id: Some(GalleryImageId(1)),
                    url: "https://example.com/gallery/1".to_string(),
                    raw_url: "https://example.com/gallery/1/raw".to_string(),
                    featured: false,
                    name: None,
                    description: None,
                    created: now,
                    ordering: 0,
                },
                GalleryItem {
                    id: Some(GalleryImageId(2)),
                    url: "https://example.com/gallery/2".to_string(),
                    raw_url: "https://example.com/gallery/2/raw".to_string(),
                    featured: false,
                    name: None,
                    description: None,
                    created: now,
                    ordering: 1,
                },
            ],
            color: None,
            thread_id: ThreadId(3),
            monetization_status: MonetizationStatus::Monetized,
            side_types_migration_review_status:
                SideTypesMigrationReviewStatus::Reviewed,
            components: exp::ProjectQuery::default(),
            fields: HashMap::new(),
        }
    }

    fn context(project: &Project) -> ThreadIssueContext<'_> {
        ThreadIssueContext {
            project,
            versions: &[],
            disclosures: &[],
            team_members: &[],
        }
    }

    fn text_target(original: &str, suggestion: Option<&str>) -> TextTarget {
        TextTarget {
            original: original.to_string(),
            suggestion: suggestion.map(str::to_string),
        }
    }

    fn optional_text_target(
        original: Option<&str>,
        suggestion: Option<Option<&str>>,
    ) -> OptionalTextTarget {
        OptionalTextTarget {
            original: original.map(str::to_string),
            suggestion: suggestion
                .map(|suggestion| suggestion.map(str::to_string)),
        }
    }

    #[test]
    fn nullable_suggestions_round_trip() {
        let target = OptionalTextTarget {
            original: Some("Original".to_string()),
            suggestion: Some(None),
        };
        let serialized = serde_json::to_value(&target).unwrap();

        assert_eq!(
            serialized.get("suggestion"),
            Some(&serde_json::Value::Null)
        );
        assert_eq!(
            serde_json::from_value::<OptionalTextTarget>(serialized).unwrap(),
            target
        );
    }

    #[test]
    fn text_target_value_states() {
        let mut project = project();
        let target = ThreadIssueTarget::ModifyTitle(text_target(
            "Example Project",
            Some("Suggested Project"),
        ));

        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsOriginal
        );

        project.name = "Another Project".to_string();
        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::DifferentToOriginal
        );

        project.name = "Suggested Project".to_string();
        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsSuggested
        );
    }

    #[test]
    fn absent_slug_uses_an_empty_current_value() {
        let mut project = project();
        project.slug = None;
        let target = ThreadIssueTarget::ModifySlug(text_target("", None));

        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsOriginal
        );
    }

    #[test]
    fn license_value_states() {
        let mut project = project();
        let target = ThreadIssueTarget::ModifyLicense {
            license: text_target("MIT", Some("Apache-2.0")),
            url: text_target("https://example.com/license", None),
        };

        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsOriginal
        );

        project.license.id = "Apache-2.0".to_string();
        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsSuggested
        );

        project.license.url =
            Some("https://example.com/other-license".to_string());
        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::DifferentToOriginal
        );
    }

    #[test]
    fn link_value_states() {
        let mut project = project();
        let target = ThreadIssueTarget::ModifyLinks {
            links: HashMap::from([
                (
                    "source".to_string(),
                    text_target(
                        "https://example.com/source",
                        Some("https://example.com/new-source"),
                    ),
                ),
                (
                    "issues".to_string(),
                    text_target("https://example.com/issues", None),
                ),
            ]),
        };

        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsOriginal
        );

        project.link_urls.get_mut("source").unwrap().url =
            "https://example.com/new-source".to_string();
        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsSuggested
        );

        project.link_urls.get_mut("issues").unwrap().url =
            "https://example.com/other-issues".to_string();
        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::DifferentToOriginal
        );
    }

    #[test]
    fn icon_and_tag_value_states() {
        let mut project = project();
        let icon = ThreadIssueTarget::ModifyIcon {
            original_url: project.icon_url.clone(),
        };
        let tags = ThreadIssueTarget::RemoveTags {
            tags: vec!["technology".to_string(), "utility".to_string()],
        };

        assert_eq!(
            icon.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsOriginal
        );
        assert_eq!(
            tags.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsOriginal
        );

        project.icon_url = None;
        project.additional_categories.clear();
        assert_eq!(
            icon.value_state(&context(&project), false),
            ThreadIssueValueState::DifferentToOriginal
        );
        assert_eq!(
            tags.value_state(&context(&project), false),
            ThreadIssueValueState::DifferentToOriginal
        );

        project.categories.clear();
        assert_eq!(
            tags.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsSuggested
        );
    }

    #[test]
    fn gallery_addition_and_metadata_value_states() {
        let mut project = project();
        let add = ThreadIssueTarget::AddGalleryImages {
            original_count: project.gallery.len() as u32,
        };
        let metadata = ThreadIssueTarget::ModifyGalleryImage {
            image_id: GalleryImageId(1),
            original_url: "https://example.com/gallery/1".to_string(),
            name: Some(optional_text_target(None, Some(Some("Screenshot")))),
            description: Some(optional_text_target(
                None,
                Some(Some("Description")),
            )),
        };

        assert_eq!(
            add.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsOriginal
        );
        assert_eq!(
            metadata.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsOriginal
        );

        let mut added = project.gallery[0].clone();
        added.id = Some(GalleryImageId(3));
        project.gallery.push(added);
        project.gallery[0].name = Some("Screenshot".to_string());
        project.gallery[0].description = Some("Description".to_string());

        assert_eq!(
            add.value_state(&context(&project), false),
            ThreadIssueValueState::DifferentToOriginal
        );
        assert_eq!(
            metadata.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsSuggested
        );

        project.gallery[0].description = Some("Unrelated text".to_string());
        assert_eq!(
            metadata.value_state(&context(&project), false),
            ThreadIssueValueState::DifferentToOriginal
        );
    }

    #[test]
    fn all_gallery_targets_must_be_removed() {
        let mut project = project();
        let target = ThreadIssueTarget::RemoveGalleryImages {
            image_ids: vec![GalleryImageId(1), GalleryImageId(2)],
        };

        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsOriginal
        );

        project.gallery.remove(0);
        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::DifferentToOriginal
        );

        project.gallery.clear();
        assert_eq!(
            target.value_state(&context(&project), false),
            ThreadIssueValueState::SameAsSuggested
        );
    }

    #[test]
    fn disclosure_value_states() {
        let project = project();
        let mut disclosures = vec![ProjectDisclosure::Advertisements {
            note: Some("Original note".to_string()),
        }];
        let remove = ThreadIssueTarget::RemoveProjectDisclosures {
            disclosure_types: vec!["advertisements".to_string()],
        };
        let note = ThreadIssueTarget::ModifyProjectDisclosureNote {
            disclosure_type: "advertisements".to_string(),
            note: optional_text_target(
                Some("Original note"),
                Some(Some("Suggested note")),
            ),
        };
        let value_state =
            |target: &ThreadIssueTarget, disclosures: &[ProjectDisclosure]| {
                target.value_state(
                    &ThreadIssueContext {
                        project: &project,
                        versions: &[],
                        disclosures,
                        team_members: &[],
                    },
                    false,
                )
            };

        assert_eq!(
            value_state(&remove, &disclosures),
            ThreadIssueValueState::SameAsOriginal
        );
        assert_eq!(
            value_state(&note, &disclosures),
            ThreadIssueValueState::SameAsOriginal
        );

        disclosures[0] = ProjectDisclosure::Advertisements {
            note: Some("Suggested note".to_string()),
        };
        assert_eq!(
            value_state(&note, &disclosures),
            ThreadIssueValueState::SameAsSuggested
        );

        disclosures.clear();
        assert_eq!(
            value_state(&remove, &disclosures),
            ThreadIssueValueState::SameAsSuggested
        );
    }

    #[test]
    fn team_member_role_value_states() {
        let project = project();
        let mut members = vec![ThreadIssueTeamMember {
            team_id: TeamId(2),
            user_id: UserId(4),
            role: "Developer".to_string(),
        }];
        let target = ThreadIssueTarget::ModifyTeamMemberRole {
            team_id: TeamId(2),
            user_id: UserId(4),
            role: text_target("Developer", Some("Artist")),
        };
        let value_state = |members: &[ThreadIssueTeamMember]| {
            target.value_state(
                &ThreadIssueContext {
                    project: &project,
                    versions: &[],
                    disclosures: &[],
                    team_members: members,
                },
                false,
            )
        };

        assert_eq!(
            value_state(&members),
            ThreadIssueValueState::SameAsOriginal
        );
        members[0].role = "Artist".to_string();
        assert_eq!(
            value_state(&members),
            ThreadIssueValueState::SameAsSuggested
        );
        members.clear();
        assert_eq!(
            value_state(&members),
            ThreadIssueValueState::DifferentToOriginal
        );
    }

    #[test]
    fn acknowledgement_value_states() {
        let checkbox = ThreadIssueTarget::Acknowledge {
            mode: ThreadIssueAcknowledgement::Checkbox,
        };
        let reply = ThreadIssueTarget::Acknowledge {
            mode: ThreadIssueAcknowledgement::Reply,
        };

        assert_eq!(
            checkbox.value_state(&context(&project()), false),
            ThreadIssueValueState::SameAsOriginal
        );
        assert_eq!(
            checkbox.value_state(&context(&project()), true),
            ThreadIssueValueState::SameAsSuggested
        );
        assert_eq!(
            reply.value_state(&context(&project()), true),
            ThreadIssueValueState::DifferentToOriginal
        );
    }

    #[test]
    fn verdict_tracks_resolution_actions() {
        let mut project = project();
        let target = ThreadIssueTarget::ModifyTitle(text_target(
            "Example Project",
            Some("Suggested Project"),
        ));

        assert_eq!(
            target.verdict(&context(&project), true, false),
            ThreadIssueVerdict::Open
        );

        project.name = "Another Project".to_string();
        assert_eq!(
            target.verdict(&context(&project), false, false),
            ThreadIssueVerdict::Open
        );
        assert_eq!(
            target.verdict(&context(&project), true, false),
            ThreadIssueVerdict::Addressed
        );

        project.name = "Suggested Project".to_string();
        assert_eq!(
            target.verdict(&context(&project), false, false),
            ThreadIssueVerdict::Resolved
        );

        project.name = "Example Project".to_string();
        assert_eq!(
            target.verdict(&context(&project), false, true),
            ThreadIssueVerdict::Resolved
        );
    }

    #[test]
    fn gallery_and_acknowledgement_verdicts() {
        let mut project = project();
        let gallery = ThreadIssueTarget::RemoveGalleryImages {
            image_ids: vec![GalleryImageId(1)],
        };
        let checkbox = ThreadIssueTarget::Acknowledge {
            mode: ThreadIssueAcknowledgement::Checkbox,
        };
        let reply = ThreadIssueTarget::Acknowledge {
            mode: ThreadIssueAcknowledgement::Reply,
        };

        assert_eq!(
            gallery.verdict(&context(&project), true, false),
            ThreadIssueVerdict::Addressed
        );
        project.gallery.remove(0);
        assert_eq!(
            gallery.verdict(&context(&project), false, false),
            ThreadIssueVerdict::Resolved
        );
        assert_eq!(
            checkbox.verdict(&context(&project), true, false),
            ThreadIssueVerdict::Resolved
        );
        assert_eq!(
            reply.verdict(&context(&project), true, false),
            ThreadIssueVerdict::Addressed
        );
    }
}
