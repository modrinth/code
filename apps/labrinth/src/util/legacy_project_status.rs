use crate::models::v3::projects::{
    ProjectReviewStatus, ProjectStatus, ProjectVisibility,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyProjectStatusFields {
    pub status: ProjectStatus,
    pub requested_status: Option<ProjectStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyStatusMeaning {
    pub visibility: Option<ProjectVisibility>,
    pub review_status: ProjectReviewStatus,
    pub archived: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LegacyProjectStatusError {
    #[error(
        "cannot decode legacy status `scheduled` without scheduling information"
    )]
    Scheduled,
    #[error("cannot decode legacy status `unknown`")]
    Unknown,
    #[error("invalid legacy requested status `{0}`")]
    InvalidRequestedStatus(ProjectStatus),
}

/// Encodes native state while retaining its audience in the requested status.
pub fn encode(
    visibility: ProjectVisibility,
    review_status: ProjectReviewStatus,
) -> LegacyProjectStatusFields {
    let requested_status = match visibility {
        ProjectVisibility::Listed => ProjectStatus::Approved,
        ProjectVisibility::Unlisted => ProjectStatus::Unlisted,
        ProjectVisibility::Private => ProjectStatus::Private,
    };
    let status = match review_status {
        ProjectReviewStatus::Draft => ProjectStatus::Draft,
        ProjectReviewStatus::InReview => ProjectStatus::Processing,
        ProjectReviewStatus::Approved => requested_status,
        ProjectReviewStatus::Rejected => ProjectStatus::Rejected,
        ProjectReviewStatus::Withheld => ProjectStatus::Withheld,
    };
    LegacyProjectStatusFields {
        status,
        requested_status: Some(requested_status),
    }
}

/// Decodes historical state without inventing an audience for review-only statuses.
pub fn decode(
    status: ProjectStatus,
) -> Result<LegacyStatusMeaning, LegacyProjectStatusError> {
    let (visibility, review_status, archived) = match status {
        ProjectStatus::Approved => (
            Some(ProjectVisibility::Listed),
            ProjectReviewStatus::Approved,
            false,
        ),
        ProjectStatus::Unlisted => (
            Some(ProjectVisibility::Unlisted),
            ProjectReviewStatus::Approved,
            false,
        ),
        ProjectStatus::Private => (
            Some(ProjectVisibility::Private),
            ProjectReviewStatus::Approved,
            false,
        ),
        ProjectStatus::Archived => (
            Some(ProjectVisibility::Listed),
            ProjectReviewStatus::Approved,
            true,
        ),
        ProjectStatus::Draft => (None, ProjectReviewStatus::Draft, false),
        ProjectStatus::Processing => {
            (None, ProjectReviewStatus::InReview, false)
        }
        ProjectStatus::Rejected => (None, ProjectReviewStatus::Rejected, false),
        ProjectStatus::Withheld => (None, ProjectReviewStatus::Withheld, false),
        ProjectStatus::Scheduled => {
            return Err(LegacyProjectStatusError::Scheduled);
        }
        ProjectStatus::Unknown => {
            return Err(LegacyProjectStatusError::Unknown);
        }
    };
    Ok(LegacyStatusMeaning {
        visibility,
        review_status,
        archived,
    })
}

/// Resolves only missing visibility; a definite main status takes precedence.
pub fn decode_fields(
    status: ProjectStatus,
    requested_status: Option<ProjectStatus>,
) -> Result<LegacyStatusMeaning, LegacyProjectStatusError> {
    let mut meaning = decode(status)?;
    if meaning.visibility.is_none() {
        meaning.visibility = requested_status
            .map(decode_requested_visibility)
            .transpose()?;
    }
    Ok(meaning)
}

pub fn decode_requested_visibility(
    status: ProjectStatus,
) -> Result<ProjectVisibility, LegacyProjectStatusError> {
    match status {
        ProjectStatus::Approved => Ok(ProjectVisibility::Listed),
        ProjectStatus::Unlisted => Ok(ProjectVisibility::Unlisted),
        ProjectStatus::Private => Ok(ProjectVisibility::Private),
        _ => Err(LegacyProjectStatusError::InvalidRequestedStatus(status)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUDIENCES: [(ProjectVisibility, ProjectStatus); 3] = [
        (ProjectVisibility::Listed, ProjectStatus::Approved),
        (ProjectVisibility::Unlisted, ProjectStatus::Unlisted),
        (ProjectVisibility::Private, ProjectStatus::Private),
    ];
    const REVIEW_ONLY: [(ProjectStatus, ProjectReviewStatus); 4] = [
        (ProjectStatus::Draft, ProjectReviewStatus::Draft),
        (ProjectStatus::Processing, ProjectReviewStatus::InReview),
        (ProjectStatus::Rejected, ProjectReviewStatus::Rejected),
        (ProjectStatus::Withheld, ProjectReviewStatus::Withheld),
    ];

    #[test]
    fn historical_thread_payloads_keep_legacy_statuses() {
        for status in ["archived", "scheduled", "unknown", "processing"] {
            let value = serde_json::json!({
                "type": "status_change",
                "old_status": status,
                "new_status": "approved",
            });
            let message: crate::models::threads::MessageBody =
                serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(message).unwrap(), value);
            let message: crate::models::v2::threads::LegacyMessageBody =
                serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(message).unwrap(), value);
        }
        let value = serde_json::json!({
            "type": "auto_approval",
            "old_status": "processing",
            "new_status": "unlisted",
        });
        let message: crate::models::threads::MessageBody =
            serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(message).unwrap(), value);
    }

    #[test]
    fn all_native_state_pairs_roundtrip() {
        for (visibility, requested_status) in AUDIENCES {
            for (review_status, expected_status) in [
                (ProjectReviewStatus::Draft, ProjectStatus::Draft),
                (ProjectReviewStatus::InReview, ProjectStatus::Processing),
                (ProjectReviewStatus::Approved, requested_status),
                (ProjectReviewStatus::Rejected, ProjectStatus::Rejected),
                (ProjectReviewStatus::Withheld, ProjectStatus::Withheld),
            ] {
                let fields = encode(visibility, review_status);
                assert_eq!(fields.status, expected_status);
                assert_eq!(fields.requested_status, Some(requested_status));
                assert_eq!(
                    decode_fields(fields.status, fields.requested_status),
                    Ok(LegacyStatusMeaning {
                        visibility: Some(visibility),
                        review_status,
                        archived: false,
                    })
                );
            }
        }
    }

    #[test]
    fn historical_review_statuses_preserve_missing_visibility() {
        for (status, review_status) in REVIEW_ONLY {
            let expected = LegacyStatusMeaning {
                visibility: None,
                review_status,
                archived: false,
            };
            assert_eq!(decode(status), Ok(expected));
            assert_eq!(decode_fields(status, None), Ok(expected));
            for (visibility, requested_status) in AUDIENCES {
                assert_eq!(
                    decode_fields(status, Some(requested_status)),
                    Ok(LegacyStatusMeaning {
                        visibility: Some(visibility),
                        ..expected
                    })
                );
            }
        }
    }

    #[test]
    fn definite_visibility_wins_over_unused_requested_status() {
        for (visibility, status) in AUDIENCES {
            for requested_status in [
                None,
                Some(ProjectStatus::Private),
                Some(ProjectStatus::Approved),
                Some(ProjectStatus::Archived),
                Some(ProjectStatus::Unknown),
            ] {
                assert_eq!(
                    decode_fields(status, requested_status),
                    Ok(LegacyStatusMeaning {
                        visibility: Some(visibility),
                        review_status: ProjectReviewStatus::Approved,
                        archived: false,
                    })
                );
            }
        }
    }

    #[test]
    fn archived_preserves_archival_information() {
        let expected = LegacyStatusMeaning {
            visibility: Some(ProjectVisibility::Listed),
            review_status: ProjectReviewStatus::Approved,
            archived: true,
        };
        assert_eq!(decode(ProjectStatus::Archived), Ok(expected));
        for requested_status in [
            None,
            Some(ProjectStatus::Private),
            Some(ProjectStatus::Unknown),
        ] {
            assert_eq!(
                decode_fields(ProjectStatus::Archived, requested_status),
                Ok(expected)
            );
        }
    }

    #[test]
    fn unsupported_main_statuses_have_explicit_errors() {
        for (status, error) in [
            (
                ProjectStatus::Scheduled,
                LegacyProjectStatusError::Scheduled,
            ),
            (ProjectStatus::Unknown, LegacyProjectStatusError::Unknown),
        ] {
            assert_eq!(decode(status), Err(error));
            assert_eq!(decode_fields(status, None), Err(error));
            assert_eq!(
                decode_fields(status, Some(ProjectStatus::Approved)),
                Err(error)
            );
        }
    }

    #[test]
    fn requested_visibility_accepts_only_audience_statuses() {
        for (visibility, status) in AUDIENCES {
            assert_eq!(decode_requested_visibility(status), Ok(visibility));
        }
        for status in [
            ProjectStatus::Archived,
            ProjectStatus::Draft,
            ProjectStatus::Processing,
            ProjectStatus::Rejected,
            ProjectStatus::Withheld,
            ProjectStatus::Scheduled,
            ProjectStatus::Unknown,
        ] {
            let error =
                LegacyProjectStatusError::InvalidRequestedStatus(status);
            assert_eq!(decode_requested_visibility(status), Err(error));
            for (main_status, _) in REVIEW_ONLY {
                assert_eq!(
                    decode_fields(main_status, Some(status)),
                    Err(error)
                );
            }
        }
    }
}
