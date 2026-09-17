use crate::models::ids::{GalleryImageId, ThreadIssueId};
use crate::models::projects::Project;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Has the user changed the affected project part since this issue was
    /// added?
    pub value_state: ThreadIssueValueState,
    /// Final derived verdict of this issue.
    pub verdict: ThreadIssueVerdict,
}

/// What part of a project must change for a [`ThreadIssue`] to be resolved?
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ThreadIssueTarget {
    Title(TextTarget),
    Slug(TextTarget),
    Summary(TextTarget),
    Description(TextTarget),
    License {
        license: TextTarget,
        url: TextTarget,
    },
    Icon {
        original_url: Option<String>,
    },
    Tags {
        original: Vec<String>,
    },
    Links {
        links: HashMap<String, TextTarget>,
    },
    RemoveGalleryImages {
        originals: Vec<(GalleryImageId, String)>,
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
        project: &Project,
        user_addressed: bool,
        moderator_verified: bool,
    ) -> ThreadIssueVerdict {
        if moderator_verified {
            return ThreadIssueVerdict::Resolved;
        }

        match (self.value_state(project, user_addressed), user_addressed) {
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
        project: &Project,
        user_addressed: bool,
    ) -> ThreadIssueValueState {
        let state = match self {
            Self::Title(target) => value_state(target, &project.name),
            Self::Slug(target) => {
                value_state(target, project.slug.as_deref().unwrap_or_default())
            }
            Self::Summary(target) => value_state(target, &project.summary),
            Self::Description(target) => {
                value_state(target, &project.description)
            }
            Self::License { license, url } => {
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
            Self::Icon { original_url } => {
                if project.icon_url == *original_url {
                    ThreadIssueValueState::SameAsOriginal
                } else {
                    ThreadIssueValueState::DifferentToOriginal
                }
            }
            Self::Tags { original } => {
                let current = project
                    .categories
                    .iter()
                    .chain(&project.additional_categories)
                    .cloned()
                    .collect::<Vec<_>>();
                if current == *original {
                    ThreadIssueValueState::SameAsOriginal
                } else {
                    ThreadIssueValueState::DifferentToOriginal
                }
            }
            Self::Links { links } => {
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
            Self::RemoveGalleryImages { originals } => {
                let remaining = originals
                    .iter()
                    .filter(|(id, _)| {
                        project
                            .gallery
                            .iter()
                            .any(|image| image.id.as_ref() == Some(id))
                    })
                    .count();

                if remaining == 0 {
                    ThreadIssueValueState::SameAsSuggested
                } else if remaining == originals.len() {
                    ThreadIssueValueState::SameAsOriginal
                } else {
                    ThreadIssueValueState::DifferentToOriginal
                }
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
        };

        state
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
    pub fn from(
        data: crate::database::models::DBThreadIssue,
        value_state: ThreadIssueValueState,
    ) -> Self {
        Self {
            id: data.id.into(),
            what: data.what,
            why: data.why,
            user_addressed: data.user_addressed,
            moderator_verified: data.moderator_verified,
            value_state,
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

    fn text_target(original: &str, suggestion: Option<&str>) -> TextTarget {
        TextTarget {
            original: original.to_string(),
            suggestion: suggestion.map(str::to_string),
        }
    }

    #[test]
    fn text_target_value_states() {
        let mut project = project();
        let target = ThreadIssueTarget::Title(text_target(
            "Example Project",
            Some("Suggested Project"),
        ));

        assert_eq!(
            target.value_state(&project, false),
            ThreadIssueValueState::SameAsOriginal
        );

        project.name = "Another Project".to_string();
        assert_eq!(
            target.value_state(&project, false),
            ThreadIssueValueState::DifferentToOriginal
        );

        project.name = "Suggested Project".to_string();
        assert_eq!(
            target.value_state(&project, false),
            ThreadIssueValueState::SameAsSuggested
        );
    }

    #[test]
    fn absent_slug_uses_an_empty_current_value() {
        let mut project = project();
        project.slug = None;
        let target = ThreadIssueTarget::Slug(text_target("", None));

        assert_eq!(
            target.value_state(&project, false),
            ThreadIssueValueState::SameAsOriginal
        );
    }

    #[test]
    fn license_value_states() {
        let mut project = project();
        let target = ThreadIssueTarget::License {
            license: text_target("MIT", Some("Apache-2.0")),
            url: text_target("https://example.com/license", None),
        };

        assert_eq!(
            target.value_state(&project, false),
            ThreadIssueValueState::SameAsOriginal
        );

        project.license.id = "Apache-2.0".to_string();
        assert_eq!(
            target.value_state(&project, false),
            ThreadIssueValueState::SameAsSuggested
        );

        project.license.url =
            Some("https://example.com/other-license".to_string());
        assert_eq!(
            target.value_state(&project, false),
            ThreadIssueValueState::DifferentToOriginal
        );
    }

    #[test]
    fn link_value_states() {
        let mut project = project();
        let target = ThreadIssueTarget::Links {
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
            target.value_state(&project, false),
            ThreadIssueValueState::SameAsOriginal
        );

        project.link_urls.get_mut("source").unwrap().url =
            "https://example.com/new-source".to_string();
        assert_eq!(
            target.value_state(&project, false),
            ThreadIssueValueState::SameAsSuggested
        );

        project.link_urls.get_mut("issues").unwrap().url =
            "https://example.com/other-issues".to_string();
        assert_eq!(
            target.value_state(&project, false),
            ThreadIssueValueState::DifferentToOriginal
        );
    }

    #[test]
    fn icon_and_tag_value_states() {
        let mut project = project();
        let icon = ThreadIssueTarget::Icon {
            original_url: project.icon_url.clone(),
        };
        let tags = ThreadIssueTarget::Tags {
            original: vec!["technology".to_string(), "utility".to_string()],
        };

        assert_eq!(
            icon.value_state(&project, false),
            ThreadIssueValueState::SameAsOriginal
        );
        assert_eq!(
            tags.value_state(&project, false),
            ThreadIssueValueState::SameAsOriginal
        );

        project.icon_url = None;
        project.additional_categories.clear();
        assert_eq!(
            icon.value_state(&project, false),
            ThreadIssueValueState::DifferentToOriginal
        );
        assert_eq!(
            tags.value_state(&project, false),
            ThreadIssueValueState::DifferentToOriginal
        );
    }

    #[test]
    fn all_gallery_targets_must_be_removed() {
        let mut project = project();
        let target = ThreadIssueTarget::RemoveGalleryImages {
            originals: vec![
                (
                    GalleryImageId(1),
                    "https://example.com/gallery/1".to_string(),
                ),
                (
                    GalleryImageId(2),
                    "https://example.com/gallery/2".to_string(),
                ),
            ],
        };

        assert_eq!(
            target.value_state(&project, false),
            ThreadIssueValueState::SameAsOriginal
        );

        project.gallery.remove(0);
        assert_eq!(
            target.value_state(&project, false),
            ThreadIssueValueState::DifferentToOriginal
        );

        project.gallery.clear();
        assert_eq!(
            target.value_state(&project, false),
            ThreadIssueValueState::SameAsSuggested
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
            checkbox.value_state(&project(), false),
            ThreadIssueValueState::SameAsOriginal
        );
        assert_eq!(
            checkbox.value_state(&project(), true),
            ThreadIssueValueState::SameAsSuggested
        );
        assert_eq!(
            reply.value_state(&project(), true),
            ThreadIssueValueState::DifferentToOriginal
        );
    }

    #[test]
    fn verdict_tracks_resolution_actions() {
        let mut project = project();
        let target = ThreadIssueTarget::Title(text_target(
            "Example Project",
            Some("Suggested Project"),
        ));

        assert_eq!(
            target.verdict(&project, true, false),
            ThreadIssueVerdict::Open
        );

        project.name = "Another Project".to_string();
        assert_eq!(
            target.verdict(&project, false, false),
            ThreadIssueVerdict::Open
        );
        assert_eq!(
            target.verdict(&project, true, false),
            ThreadIssueVerdict::Addressed
        );

        project.name = "Suggested Project".to_string();
        assert_eq!(
            target.verdict(&project, false, false),
            ThreadIssueVerdict::Resolved
        );

        project.name = "Example Project".to_string();
        assert_eq!(
            target.verdict(&project, false, true),
            ThreadIssueVerdict::Resolved
        );
    }

    #[test]
    fn gallery_and_acknowledgement_verdicts() {
        let mut project = project();
        let gallery = ThreadIssueTarget::RemoveGalleryImages {
            originals: vec![(
                GalleryImageId(1),
                "https://example.com/gallery/1".to_string(),
            )],
        };
        let checkbox = ThreadIssueTarget::Acknowledge {
            mode: ThreadIssueAcknowledgement::Checkbox,
        };
        let reply = ThreadIssueTarget::Acknowledge {
            mode: ThreadIssueAcknowledgement::Reply,
        };

        assert_eq!(
            gallery.verdict(&project, true, false),
            ThreadIssueVerdict::Addressed
        );
        project.gallery.remove(0);
        assert_eq!(
            gallery.verdict(&project, false, false),
            ThreadIssueVerdict::Resolved
        );
        assert_eq!(
            checkbox.verdict(&project, true, false),
            ThreadIssueVerdict::Resolved
        );
        assert_eq!(
            reply.verdict(&project, true, false),
            ThreadIssueVerdict::Addressed
        );
    }
}
