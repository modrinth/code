use actix_http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, ResponseError, put, web};
use rust_decimal::Decimal;
use validator::Validate;

use crate::{
    auth::get_user_from_headers,
    database::{
        PgPool,
        models::{
            self, DBUser, project_item::ProjectBuilder,
            thread_item::ThreadBuilder,
        },
        redis::RedisPool,
    },
    models::{
        exp::{self},
        ids::ProjectId,
        pats::Scopes,
        projects::{MonetizationStatus, ProjectStatus},
        teams::ProjectPermissions,
        threads::ThreadType,
        v3::user_limits::UserLimits,
    },
    queue::session::AuthQueue,
    routes::ApiError,
    util::{error::Context, validate::validation_errors_to_string},
};

// pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
//     cfg.service(create);
// }

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(create);
}

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error("project limit reached")]
    LimitReached,
    #[error("invalid component kinds")]
    ComponentKinds(exp::ComponentKindsError),
    #[error("failed to validate request: {0}")]
    Validation(String),
    #[error("slug collision")]
    SlugCollision,
    #[error(transparent)]
    Api(#[from] ApiError),
}

impl CreateError {
    pub fn as_api_error(&self) -> crate::models::error::ApiError<'_> {
        match self {
            Self::LimitReached => crate::models::error::ApiError {
                error: "limit_reached",
                description: self.to_string(),
                details: None,
            },
            Self::ComponentKinds(err) => crate::models::error::ApiError {
                error: "component_kinds",
                description: format!("{self}: {err}"),
                details: Some(
                    serde_json::to_value(err)
                        .expect("should never fail to serialize"),
                ),
            },
            Self::Validation(_) => crate::models::error::ApiError {
                error: "validation",
                description: self.to_string(),
                details: None,
            },
            Self::SlugCollision => crate::models::error::ApiError {
                error: "slug_collision",
                description: self.to_string(),
                details: None,
            },
            Self::Api(err) => err.as_api_error(),
        }
    }
}

impl ResponseError for CreateError {
    fn status_code(&self) -> actix_http::StatusCode {
        match self {
            Self::LimitReached => StatusCode::BAD_REQUEST,
            Self::ComponentKinds(_) => StatusCode::BAD_REQUEST,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::SlugCollision => StatusCode::BAD_REQUEST,
            Self::Api(err) => err.status_code(),
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.as_api_error())
    }
}

/// Creates a new project with the given components.
///
/// Components must include `base` ([`exp::base::Project`]), and at least one
/// other component.
#[utoipa::path]
#[put("/project")]
pub async fn create(
    req: HttpRequest,
    db: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    web::Json(details): web::Json<exp::ProjectCreate>,
) -> Result<web::Json<ProjectId>, CreateError> {
    // check that the user can make a project
    let (_, user) = get_user_from_headers(
        &req,
        &**db,
        &redis,
        &session_queue,
        Scopes::PROJECT_CREATE,
    )
    .await
    .map_err(ApiError::from)?;

    let limits = UserLimits::get_for_projects(&user, &db)
        .await
        .map_err(ApiError::from)?;
    if limits.current >= limits.max {
        return Err(CreateError::LimitReached);
    }

    // check if the given details are valid

    exp::component_kinds_valid(&details.component_kinds())
        .map_err(CreateError::ComponentKinds)?;

    details.validate().map_err(|err| {
        CreateError::Validation(validation_errors_to_string(err, None))
    })?;

    // get component-specific data
    // use struct destructor syntax, so we get a compile error
    // if we add a new field and don't add it here
    let exp::ProjectCreate {
        base,
        minecraft_mod,
        minecraft_server,
        minecraft_java_server,
        minecraft_bedrock_server,
    } = details;

    // check if this won't conflict with an existing project

    let mut txn = db
        .begin()
        .await
        .wrap_internal_err("failed to begin transaction")?;

    let same_slug_record = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM mods WHERE text_id_lower = $1)",
        base.slug.to_lowercase()
    )
    .fetch_one(&mut txn)
    .await
    .wrap_internal_err("failed to query if slug already exists")?;

    if same_slug_record.exists.unwrap_or(false) {
        return Err(CreateError::SlugCollision);
    }

    // create project and supporting records in db

    let team_id = {
        // TODO organization
        let members = vec![models::team_item::TeamMemberBuilder {
            user_id: user.id.into(),
            role: crate::models::teams::DEFAULT_ROLE.to_owned(),
            is_owner: true,
            permissions: ProjectPermissions::all(),
            organization_permissions: None,
            accepted: true,
            payouts_split: Decimal::ONE_HUNDRED,
            ordering: 0,
        }];
        let team = models::team_item::TeamBuilder { members };
        team.insert(&mut txn)
            .await
            .wrap_internal_err("failed to insert team")?
    };

    let project_id: ProjectId = models::generate_project_id(&mut txn)
        .await
        .wrap_internal_err("failed to generate project ID")?
        .into();

    let project_builder = ProjectBuilder {
        project_id: project_id.into(),
        team_id,
        organization_id: None, // todo
        name: base.name.clone(),
        summary: base.summary.clone(),
        description: base.description.clone(),
        icon_url: None,
        raw_icon_url: None,
        license_url: None,
        categories: vec![],
        additional_categories: vec![],
        initial_versions: vec![],
        status: ProjectStatus::Draft,
        requested_status: Some(ProjectStatus::Approved),
        license: "LicenseRef-Unknown".into(),
        slug: Some(base.slug.clone()),
        link_urls: vec![],
        gallery_items: vec![],
        color: None,
        // TODO: what if we don't monetize server listing projects?
        monetization_status: MonetizationStatus::Monetized,
        // components
        components: exp::ProjectSerial {
            minecraft_mod: minecraft_mod
                .map(exp::ProjectComponent::into_serial),
            minecraft_server: minecraft_server
                .map(exp::ProjectComponent::into_serial),
            minecraft_java_server: minecraft_java_server
                .map(exp::ProjectComponent::into_serial),
            minecraft_bedrock_server: minecraft_bedrock_server
                .map(exp::ProjectComponent::into_serial),
        },
    };

    project_builder
        .insert(&mut txn)
        .await
        .wrap_internal_err("failed to insert project")?;
    DBUser::clear_project_cache(&[user.id.into()], &redis)
        .await
        .wrap_internal_err("failed to clear user project cache")?;

    ThreadBuilder {
        type_: ThreadType::Project,
        members: vec![],
        project_id: Some(project_id.into()),
        report_id: None,
    }
    .insert(&mut txn)
    .await
    .wrap_internal_err("failed to insert thread")?;

    // and commit!

    txn.commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(web::Json(project_id))
}
