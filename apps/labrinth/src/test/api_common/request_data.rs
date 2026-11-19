// The structures for project/version creation.
// These are created differently, but are essentially the same between versions.

use crate::util::actix::MultipartSegment;

use crate::test::dummy_data::TestFile;

pub struct ProjectCreationRequestData {
    pub slug: String,
    pub jar: Option<TestFile>,
    pub segment_data: Vec<MultipartSegment>,
}

pub struct VersionCreationRequestData {
    pub version: String,
    pub jar: Option<TestFile>,
    pub segment_data: Vec<MultipartSegment>,
}

pub struct ImageData {
    pub filename: String,
    pub extension: String,
    pub icon: Vec<u8>,
}

/// Translate the legacy common-test contract only when targeting v3 routes.
pub fn v3_project_patch(mut patch: serde_json::Value) -> serde_json::Value {
    let Some(fields) = patch.as_object_mut() else {
        return patch;
    };
    if let Some(status) = fields.remove("status") {
        match status.as_str() {
            Some("approved" | "rejected" | "withheld") => {
                fields.insert("review_outcome".into(), status);
                fields.insert("in_review".into(), false.into());
            }
            Some("processing") => {
                fields.insert("in_review".into(), true.into());
            }
            Some("draft") => {
                fields.insert("in_review".into(), false.into());
            }
            _ => {
                fields.insert("visibility".into(), status);
            }
        }
    }
    if let Some(status) = fields.remove("requested_status") {
        let visibility = if status == "approved" {
            "public".into()
        } else {
            status
        };
        fields.insert("visibility".into(), visibility);
    }
    patch
}

#[cfg(test)]
mod tests {
    use super::v3_project_patch;
    use serde_json::json;

    #[test]
    fn maps_legacy_review_requests_to_v3() {
        assert_eq!(
            v3_project_patch(
                json!({ "status": "processing", "requested_status": "unlisted" })
            ),
            json!({ "in_review": true, "visibility": "unlisted" }),
        );
        assert_eq!(
            v3_project_patch(json!({ "status": "approved" })),
            json!({ "review_outcome": "approved", "in_review": false }),
        );
        assert_eq!(
            v3_project_patch(json!({ "status": "draft" })),
            json!({ "in_review": false }),
        );
    }

    #[test]
    fn preserves_canonical_v3_requests() {
        let patch = json!({ "visibility": "public", "in_review": false, "review_outcome": "approved" });
        assert_eq!(v3_project_patch(patch.clone()), patch);
    }
}
