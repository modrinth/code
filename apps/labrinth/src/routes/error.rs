use crate::database::models::loader_fields::VersionFieldParseError;
use crate::file_hosting::FileHostingError;
use crate::models::error::AsApiError;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use ariadne::i18n_enum;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Environment Error")]
    Env(#[from] dotenvy::Error),
    #[error("Error while uploading file: {0}")]
    FileHosting(#[from] FileHostingError),
    #[error("Database Error: {0}")]
    Database(#[from] crate::database::models::DatabaseError),
    #[error("Database Error: {0}")]
    SqlxDatabase(#[from] sqlx::Error),
    #[error("Database Error: {0}")]
    RedisDatabase(#[from] redis::RedisError),
    #[error("Clickhouse Error: {0}")]
    Clickhouse(#[from] clickhouse::error::Error),
    #[error("Internal server error: {0}")]
    Xml(serde_xml_rs::Error),
    #[error("Deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Authentication Error: {0}")]
    Authentication(#[from] crate::auth::AuthenticationError),
    #[error("Authentication Error: {0}")]
    SpecificAuthentication(#[from] SpecificAuthenticationError),
    // TODO: Use an I18nEnum instead of a String
    #[error("Invalid Input: {0}")]
    InvalidInput(String),
    #[error("Invalid Input: {0}")]
    InvalidLoaderField(#[from] VersionFieldParseError),
    // TODO: Perhaps remove this in favor of InvalidInput?
    #[error("Error while validating input: {0}")]
    Validation(String),
    #[error("Search Error: {0}")]
    Search(#[from] meilisearch_sdk::errors::Error),
    #[error("Indexing Error: {0}")]
    Indexing(#[from] crate::search::indexing::IndexingError),
    // TODO: Use an I18nEnum instead of a String
    #[error("Payments Error: {0}")]
    Payments(String),
    // TODO: Use an I18nEnum instead of a String
    #[error("Discord Error: {0}")]
    Discord(String),
    #[error("Slack Webhook Error: Error while sending projects webhook")]
    Slack,
    #[error("Captcha Error. Try resubmitting the form.")]
    Turnstile,
    #[error("Error while decoding Base62: {0}")]
    Decoding(#[from] ariadne::ids::DecodingError),
    #[error("Image Parsing Error: {0}")]
    ImageParse(#[from] image::ImageError),
    #[error("Password Hashing Error: {0}")]
    PasswordHashing(#[from] argon2::password_hash::Error),
    #[error("{0}")]
    Mail(#[from] crate::queue::email::MailError),
    #[error("Error while rerouting request: {0}")]
    Reroute(#[from] reqwest::Error),
    #[error("Unable to read Zip Archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Resource not found")]
    NotFound,
    #[error("The requested route does not exist")]
    RouteNotFound,
    // TODO: Use an I18nEnum instead of a String
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("External tax compliance API Error")]
    TaxComplianceApi,
    #[error(
        "You are being rate-limited. Please wait {0} milliseconds. 0/{1} remaining."
    )]
    RateLimitError(u128, u32),
    #[error("Error while interacting with payment processor: {0}")]
    Stripe(#[from] stripe::StripeError),
}

i18n_enum!(
    ApiError,
    root_key: "labrinth.error",
    Env(..) => "environment_error",
    FileHosting(cause) => "file_hosting_error",
    Database(cause) => "database_error",
    SqlxDatabase(cause) => "database_error",
    RedisDatabase(cause) => "database_error",
    Clickhouse(cause) => "clickhouse_error",
    Xml(cause) => "xml_error",
    Json(cause) => "json_error",
    Authentication(cause) => "unauthorized",
    SpecificAuthentication(cause) => "unauthorized",
    InvalidInput(cause) => "invalid_input",
    InvalidLoaderField(cause) => "invalid_input",
    Validation(cause) => "invalid_input.validation",
    Search(cause) => "search_error",
    Indexing(cause) => "indexing_error",
    Payments(cause) => "payments_error",
    Discord(cause) => "discord_error",
    Slack! => "slack_error",
    Turnstile! => "turnstile_error",
    Decoding(cause) => "decoding_error",
    ImageParse(cause) => "invalid_image",
    PasswordHashing(cause) => "password_hashing_error",
    Mail(transparent cause) => "mail_error",
    Reroute(cause) => "reroute_error",
    Zip(cause) => "zip_error",
    Io(cause) => "io_error",
    NotFound! => "not_found",
    RouteNotFound! => "not_found.route",
    Conflict(cause) => "conflict",
    TaxComplianceApi! => "tax_compliance_api_error",
    RateLimitError(wait_ms, total_allowed_requests) => "ratelimit_error",
    Stripe(cause) => "stripe_error",
);

#[derive(Clone, Debug, Error)]
pub enum SpecificAuthenticationError {
    #[error("You do not have permission to refund a subscription!")]
    Refund,
    #[error("Invalid master key")]
    InvalidMasterKey,
    #[error(
        "You don't have sufficient permissions to interact with this OAuth application"
    )]
    InsufficientOAuthPermissions,
    #[error("You don't have permission to set this status!")]
    SetStatus,
    #[error(
        "The password change flow code is invalid or has expired. Did you copy it promptly and correctly?"
    )]
    InvalidFlowCode,
    #[error("You must specify the old password to change your password!")]
    OldPasswordNotSpecified,
    #[error("You are not authorized to upload images for this project")]
    UploadProjectImages,
    #[error("You are not authorized to upload images for this version")]
    UploadVersionImages,
    #[error("You are not authorized to upload images for this thread message")]
    UploadThreadMessageImages,
    #[error("You are not authorized to upload images for this report")]
    UploadReportImages,
    #[error("You are not authorized to read this notification!")]
    NotificationRead,
    #[error("You are not authorized to delete this notification!")]
    NotificationDelete,
    #[error(
        "You do not have permission to see the OAuth clients of this user!"
    )]
    SeeOAuthClients,
    #[error(
        "You do not have the permissions to edit the slug of this organization!"
    )]
    EditOrgSlug,
    #[error(
        "You do not have the permissions to edit the description of this organization!"
    )]
    EditOrgDescription,
    #[error(
        "You do not have the permissions to edit the name of this organization!"
    )]
    EditOrgName,
    #[error("You do not have permission to edit this organization!")]
    EditOrg,
    #[error("You don't have permission to delete this organization!")]
    DeleteOrg,
    #[error(
        "You need to be an owner of a project to add it to an organization!"
    )]
    NotProjectOwnerForAddToOrg,
    #[error("You do not have permission to add projects to this organization!")]
    AddToOrg,
    #[error("You don't have permission to edit this organization's icon.")]
    EditOrgIcon,
    #[error("You do not have permission to edit this project!")]
    EditProject,
    #[error(
        "You do not have the permissions to edit the name of this project!"
    )]
    EditProjectName,
    #[error(
        "You do not have the permissions to edit the summary of this project!"
    )]
    EditProjectSummary,
    #[error(
        "You do not have the permissions to edit the status of this project!"
    )]
    EditProjectStatus,
    #[error("You don't have permission to set this status!")]
    RestrictedProjectStatus,
    #[error(
        "You do not have the permissions to edit the requested status of this project!"
    )]
    EditProjectRequestedStatus,
    #[error(
        "You do not have the permissions to edit the license URL of this project!"
    )]
    EditProjectLicenseUrl,
    #[error(
        "You do not have the permissions to edit the slug of this project!"
    )]
    EditProjectSlug,
    #[error(
        "You do not have the permissions to edit the license of this project!"
    )]
    EditProjectLicense,
    #[error(
        "You do not have the permissions to edit the links of this project!"
    )]
    EditProjectLinks,
    #[error(
        "You do not have the permissions to edit the moderation message of this project!"
    )]
    EditProjectModerationMessage,
    #[error(
        "You do not have the permissions to edit the moderation message body of this project!"
    )]
    EditProjectModerationMessageBody,
    #[error(
        "You do not have the permissions to edit the description (body) of this project!"
    )]
    EditProjectDescription,
    #[error(
        "You do not have the permissions to edit the monetization status of this project!"
    )]
    EditProjectMonetizationStatus,
    #[error(
        "You do not have the permissions to edit the side types migration review status of this project!"
    )]
    EditProjectSideTypesMigrationReviewStatus,
    #[error(
        "You do not have the permissions to edit the additional categories of this project!"
    )]
    EditProjectAdditionalCategories,
    #[error(
        "You do not have the permissions to edit the categories of this project!"
    )]
    EditProjectCategories,
    #[error("You do not have the permissions to bulk edit project {0}!")]
    BulkEditProject(String),
    #[error("You are not a member of project {0}!")]
    NotMemberOfProject(String),
    #[error("You don't have permission to edit this project's icon.")]
    EditProjectIcon,
    #[error("You have reached the maximum of gallery images to upload.")]
    MaximumGalleryImages,
    #[error("You don't have permission to edit this project's gallery.")]
    EditProjectGallery,
    #[error("You don't have permission to delete this project!")]
    DeleteProject,
    #[error("Unable to obtain user IP address!")]
    UnknownUserIp,
    #[error(
        "You do not have permission to upload a version for this shared instance."
    )]
    SharedInstanceUploadVersion,
    #[error("You do not have permission to edit this shared instance.")]
    EditSharedInstance,
    #[error("You do not have permission to delete this shared instance.")]
    DeleteSharedInstance,
    #[error(
        "You do not have permission to delete this shared instance version."
    )]
    DeleteSharedInstanceVersion,
    #[error("You don't have permission to invite users to this team")]
    InviteUsersToTeam,
    #[error("You don't have permission to invite users to this organization")]
    InviteUsersToOrg,
    #[error(
        "You do not have permission to give this user default project permissions."
    )]
    GiveUserDefaultProjectPermissions,
    #[error("You don't have permission to edit members of this team")]
    EditTeamMembers,
    #[error(
        "You cannot override the project permissions of the organization owner!"
    )]
    OverrideOrgOwnerDefaultProjectPermissions,
    #[error("You don't have permission to edit the ownership of this team")]
    EditTeamOwnership,
    #[error("The owner can't be removed from a team")]
    RemoveOwnerFromTeam,
    #[error("You do not have permission to remove a member from this team")]
    RemoveTeamMember,
    #[error("You do not have permission to cancel a team invite")]
    CancelTeamInvite,
    #[error(
        "You do not have permission to remove a member from this organization"
    )]
    RemoveOrgMember,
    #[error("You do not have permission to cancel an organization invite")]
    CancelOrgInvite,
    #[error("You cannot delete this message!")]
    DeleteMessage,
    #[error("You do not have permission to get a user from their email!")]
    GetUserFromEmail,
    #[error("You do not have the permissions to edit the role of this user!")]
    EditUserRole,
    #[error("You do not have the permissions to edit the badges of this user!")]
    EditUserBadges,
    #[error(
        "You do not have the permissions to edit the venmo handle of this user!"
    )]
    EditUserVenmoHandle,
    #[error("You do not have permission to edit this user!")]
    EditUser,
    #[error("You don't have permission to edit this user's icon.")]
    EditUserIcon,
    #[error("You do not have permission to delete this user!")]
    DeleteUser,
    #[error(
        "You do not have permission to see the projects this user follows!"
    )]
    SeeUserFollows,
    #[error(
        "You do not have permission to see the notifications of this user!"
    )]
    SeeUserNotifications,
    #[error("You don't have permission to delete this file!")]
    DeleteFile,
    #[error("You do not have the permissions to edit this version!")]
    EditVersion,
    #[error("You don't have permission to set the downloads of this mod")]
    SetModDownloads,
    #[error("You do not have permission to delete versions in this team")]
    DeleteVersionsInTeam,
}

i18n_enum!(
    SpecificAuthenticationError,
    root_key: "labrinth.error.unauthorized.specific",
    Refund! => "refund",
    InvalidMasterKey! => "invalid_master_key",
    InsufficientOAuthPermissions! => "insufficient_oauth_permissions",
    SetStatus! => "set_status",
    InvalidFlowCode! => "invalid_flow_code",
    OldPasswordNotSpecified! => "old_password_not_specified",
    UploadProjectImages! => "upload_project_images",
    UploadVersionImages! => "upload_version_images",
    UploadThreadMessageImages! => "upload_thread_message_images",
    UploadReportImages! => "upload_report_images",
    NotificationRead! => "notification_read",
    NotificationDelete! => "notification_delete",
    SeeOAuthClients! => "see_oauth_clients",
    EditOrgSlug! => "edit_org_slug",
    EditOrgDescription! => "edit_org_description",
    EditOrgName! => "edit_org_name",
    EditOrg! => "edit_org",
    DeleteOrg! => "delete_org",
    NotProjectOwnerForAddToOrg! => "not_project_owner_for_add_to_org",
    AddToOrg! => "add_to_org",
    EditOrgIcon! => "edit_org_icon",
    EditProject! => "edit_project",
    EditProjectName! => "edit_project_name",
    EditProjectSummary! => "edit_project_summary",
    EditProjectStatus! => "edit_project_status",
    RestrictedProjectStatus! => "restricted_project_status",
    EditProjectRequestedStatus! => "edit_project_requested_status",
    EditProjectLicenseUrl! => "edit_project_license_url",
    EditProjectSlug! => "edit_project_slug",
    EditProjectLicense! => "edit_project_license",
    EditProjectLinks! => "edit_project_links",
    EditProjectModerationMessage! => "edit_project_moderation_message",
    EditProjectModerationMessageBody! => "edit_project_moderation_message_body",
    EditProjectDescription! => "edit_project_description",
    EditProjectMonetizationStatus! => "edit_project_monetization_status",
    EditProjectSideTypesMigrationReviewStatus! => "edit_project_side_types_migration_review_status",
    EditProjectAdditionalCategories! => "edit_project_additional_categories",
    EditProjectCategories! => "edit_project_categories",
    BulkEditProject(project_name) => "bulk_edit_project",
    NotMemberOfProject(project_name) => "not_member_of_project",
    EditProjectIcon! => "edit_project_icon",
    MaximumGalleryImages! => "maximum_gallery_images",
    EditProjectGallery! => "edit_project_gallery",
    DeleteProject! => "delete_project",
    UnknownUserIp! => "unknown_user_ip",
    SharedInstanceUploadVersion! => "shared_instance_upload_version",
    EditSharedInstance! => "edit_shared_instance",
    DeleteSharedInstance! => "delete_shared_instance",
    DeleteSharedInstanceVersion! => "delete_shared_instance_version",
    InviteUsersToTeam! => "invite_users_to_team",
    InviteUsersToOrg! => "invite_users_to_org",
    GiveUserDefaultProjectPermissions! => "give_user_default_project_permissions",
    EditTeamMembers! => "edit_team_members",
    OverrideOrgOwnerDefaultProjectPermissions! => "override_org_owner_default_project_permissions",
    EditTeamOwnership! => "edit_team_ownership",
    RemoveOwnerFromTeam! => "remove_owner_from_team",
    RemoveTeamMember! => "remove_team_member",
    CancelTeamInvite! => "cancel_team_invite",
    RemoveOrgMember! => "remove_org_member",
    CancelOrgInvite! => "cancel_org_invite",
    DeleteMessage! => "delete_message",
    GetUserFromEmail! => "get_user_from_email",
    EditUserRole! => "edit_user_role",
    EditUserBadges! => "edit_user_badges",
    EditUserVenmoHandle! => "edit_user_venmo_handle",
    EditUser! => "edit_user",
    EditUserIcon! => "edit_user_icon",
    DeleteUser! => "delete_user",
    SeeUserFollows! => "see_user_follows",
    SeeUserNotifications! => "see_user_notifications",
    DeleteFile! => "delete_file",
    EditVersion! => "edit_version",
    SetModDownloads! => "set_mod_downloads",
    DeleteVersionsInTeam! => "delete_versions_in_team",
);

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Env(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Database(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::SqlxDatabase(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::RedisDatabase(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Clickhouse(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Authentication(..) => StatusCode::UNAUTHORIZED,
            ApiError::SpecificAuthentication(..) => StatusCode::UNAUTHORIZED,
            ApiError::Xml(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Json(..) => StatusCode::BAD_REQUEST,
            ApiError::Search(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Indexing(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::FileHosting(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InvalidInput(..) => StatusCode::BAD_REQUEST,
            ApiError::InvalidLoaderField(..) => StatusCode::BAD_REQUEST,
            ApiError::Validation(..) => StatusCode::BAD_REQUEST,
            ApiError::Payments(..) => StatusCode::FAILED_DEPENDENCY,
            ApiError::Discord(..) => StatusCode::FAILED_DEPENDENCY,
            ApiError::Turnstile => StatusCode::BAD_REQUEST,
            ApiError::Decoding(..) => StatusCode::BAD_REQUEST,
            ApiError::ImageParse(..) => StatusCode::BAD_REQUEST,
            ApiError::PasswordHashing(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Mail(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Reroute(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::RouteNotFound => StatusCode::NOT_FOUND,
            ApiError::Conflict(..) => StatusCode::CONFLICT,
            ApiError::TaxComplianceApi => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Zip(..) => StatusCode::BAD_REQUEST,
            ApiError::Io(..) => StatusCode::BAD_REQUEST,
            ApiError::RateLimitError(..) => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Stripe(..) => StatusCode::FAILED_DEPENDENCY,
            ApiError::Slack => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.as_api_error())
    }
}
