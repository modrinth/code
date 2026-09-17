use crate::models::ids::{GalleryImageId, ThreadIssueId};
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
    pub value_state: Option<ThreadIssueValueState>,
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
    Url(TextTarget),
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
    GalleryImage {
        originals: HashMap<GalleryImageId, String>,
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

/// Has the user changed the project part affected by a [`ThreadIssue`] since
/// this issue was added?
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

impl ThreadIssue {
    pub fn from(
        data: crate::database::models::DBThreadIssue,
        value_state: Option<ThreadIssueValueState>,
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
