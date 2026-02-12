use actix_http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, ResponseError, put, web};
use eyre::eyre;
use rust_decimal::Decimal;
use validator::Validate;

use crate::{
    auth::get_user_from_headers,
    database::{
        PgPool,
        models::{
            self, DBOrganization, DBTeamMember, DBUser,
            project_item::ProjectBuilder, thread_item::ThreadBuilder,
        },
        redis::RedisPool,
    },
    models::{
        exp::{self, ProjectComponentKind, component::ComponentRelationError},
        ids::ProjectId,
        pats::Scopes,
        projects::{MonetizationStatus, ProjectStatus},
        teams::{OrganizationPermissions, ProjectPermissions},
        threads::ThreadType,
        v3::user_limits::UserLimits,
    },
    queue::session::AuthQueue,
    routes::ApiError,
    util::{error::Context, validate::validation_errors_to_string},
};

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(create);
}

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error("project limit reached")]
    LimitReached,
    #[error("missing base component")]
    MissingBase,
    #[error("invalid component kinds")]
    ComponentKinds(ComponentRelationError<ProjectComponentKind>),
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
            Self::MissingBase => crate::models::error::ApiError {
                error: "missing_base",
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
            Self::LimitReached
            | Self::MissingBase
            | Self::ComponentKinds(_)
            | Self::Validation(_)
            | Self::SlugCollision => StatusCode::BAD_REQUEST,
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
#[put("")]
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

    exp::component::kinds_valid(
        &details.component_kinds(),
        &exp::PROJECT_COMPONENT_RELATIONS,
    )
    .map_err(CreateError::ComponentKinds)?;

    details.validate().map_err(|err| {
        CreateError::Validation(validation_errors_to_string(err, None))
    })?;

    // get component-specific data
    // use struct destructor syntax, so we get a compile error
    // if we add a new field and don't add it here
    let exp::base::Project {
        name,
        slug,
        summary,
        description,
        requested_status,
        organization_id,
    } = details.base.clone().ok_or(CreateError::MissingBase)?;

    // check if this won't conflict with an existing project

    let mut txn = db
        .begin()
        .await
        .wrap_internal_err("failed to begin transaction")?;

    let same_slug_record = sqlx::query!(
        "SELECT EXISTS(
            SELECT 1 FROM mods WHERE slug = $1 OR text_id_lower = $1
        )",
        slug.to_lowercase()
    )
    .fetch_one(&mut txn)
    .await
    .wrap_internal_err("failed to query if slug already exists")?;

    if same_slug_record.exists.unwrap_or(false) {
        return Err(CreateError::SlugCollision);
    }

    // create project and supporting records in db

    let team = if let Some(organization_id) = organization_id {
        let org = DBOrganization::get_id(organization_id.into(), &**db, &redis)
            .await
            .wrap_internal_err("failed to get organization")?
            .wrap_request_err("invalid organization ID")?;

        let team_member =
            DBTeamMember::get_from_user_id(org.team_id, user.id.into(), &**db)
                .await
                .wrap_internal_err(
                    "failed to get team member of user for organization",
                )?;

        let perms = OrganizationPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
        );

        if !perms
            .is_some_and(|p| p.contains(OrganizationPermissions::ADD_PROJECT))
        {
            return Err(ApiError::Auth(eyre!(
                "no permission to create projects in this organization"
            ))
            .into());
        }

        models::team_item::TeamBuilder {
            members: Vec::new(),
        }
    } else {
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

        models::team_item::TeamBuilder { members }
    };
    let team_id = team
        .insert(&mut txn)
        .await
        .wrap_internal_err("failed to insert team")?;

    let project_id: ProjectId = models::generate_project_id(&mut txn)
        .await
        .wrap_internal_err("failed to generate project ID")?
        .into();

    // TODO: special-case server projects to be unmonetized
    let monetization_status = if details.minecraft_server.is_some() {
        MonetizationStatus::ForceDemonetized
    } else {
        MonetizationStatus::Monetized
    };

    let project_builder = ProjectBuilder {
        project_id: project_id.into(),
        team_id,
        organization_id: organization_id.map(From::from),
        name: name.clone(),
        summary: summary.clone(),
        description: description.clone(),
        icon_url: None,
        raw_icon_url: None,
        license_url: None,
        categories: vec![],
        additional_categories: vec![],
        initial_versions: vec![],
        status: ProjectStatus::Draft,
        requested_status: Some(requested_status),
        license: "LicenseRef-Unknown".into(),
        slug: Some(slug.clone()),
        link_urls: vec![],
        gallery_items: vec![],
        color: None,
        monetization_status,
        components: details,
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
