use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::{ids::OrganizationId, projects::ProjectVisibility};

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct Project {
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
    /// The visibility requested by the project owner.
    #[serde(default = "default_visibility")]
    pub visibility: ProjectVisibility,
    pub requested_status: Option<crate::models::v2::projects::ProjectStatus>,
    /// What organization the project belongs to.
    pub organization_id: Option<OrganizationId>,
}

fn default_visibility() -> ProjectVisibility {
    ProjectVisibility::Public
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct Version {}
