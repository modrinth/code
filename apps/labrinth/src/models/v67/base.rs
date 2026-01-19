use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct Create {
    /// Human-readable friendly name of the project.
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    pub name: String,
    /// Slug of the project, used in vanity URLs.
    #[validate(
        length(min = 3, max = 64),
        regex(path = *crate::util::validate::RE_URL_SAFE)
    )]
    pub slug: String,
    /// Short description of the project.
    #[validate(length(min = 3, max = 255))]
    pub summary: String,
    /// A long description of the project, in markdown.
    #[validate(length(max = 65536))]
    pub description: String,
}
