use super::ApiError;
use crate::database;
use crate::database::redis::RedisPool;
use crate::models::teams::ProjectPermissions;
use crate::{
    auth::get_user_from_headers,
    database::models::user_item,
    models::{
        ids::{ProjectId, VersionId},
        pats::Scopes,
    },
    queue::session::AuthQueue,
};
use actix_web::{HttpRequest, HttpResponse, web};
use ariadne::ids::base62_impl::to_base62;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::types::PgInterval;
use std::collections::HashMap;
use std::convert::TryInto;
use std::num::NonZeroU32;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("analytics")
            .route("playtime", web::get().to(playtimes_get))
            .route("views", web::get().to(views_get))
            .route("downloads", web::get().to(downloads_get))
            .route("revenue", web::get().to(revenue_get))
            .route(
                "countries/downloads",
                web::get().to(countries_downloads_get),
            )
            .route("countries/views", web::get().to(countries_views_get)),
    );
}

/// The json data to be passed to fetch analytic data
/// Either a list of project_ids or version_ids can be used, but not both. Unauthorized projects/versions will be filtered out.
/// start_date and end_date are optional, and default to two weeks ago, and the maximum date respectively.
/// resolution_minutes is optional. This refers to the window by which we are looking (every day, every minute, etc) and defaults to 1440 (1 day)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetData {
    // only one of project_ids or version_ids should be used
    // if neither are provided, all projects the user has access to will be used
    pub project_ids: Option<String>,

    pub start_date: Option<DateTime<Utc>>, // defaults to 2 weeks ago
    pub end_date: Option<DateTime<Utc>>,   // defaults to now

    pub resolution_minutes: Option<NonZeroU32>, // defaults to 1 day. Ignored in routes that do not aggregate over a resolution (eg: /countries)
}

/// Get playtime data for a set of projects or versions
/// Data is returned as a hashmap of project/version ids to a hashmap of days to playtime data
/// eg:
/// {
///     "4N1tEhnO": {
///         "20230824": 23
///    }
///}
/// Either a list of project_ids or version_ids can be used, but not both. Unauthorized projects/versions will be filtered out.
#[derive(Serialize, Deserialize, Clone)]
pub struct FetchedPlaytime {
    pub time: u64,
    pub total_seconds: u64,
    pub loader_seconds: HashMap<String, u64>,
    pub game_version_seconds: HashMap<String, u64>,
    pub parent_seconds: HashMap<VersionId, u64>,
}
pub async fn playtimes_get(
    req: HttpRequest,
    clickhouse: web::Data<clickhouse::Client>,
    data: web::Query<GetData>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::ANALYTICS]),
    )
    .await
    .map(|x| x.1)?;

    let project_ids = data
        .project_ids
        .as_ref()
        .map(|ids| serde_json::from_str::<Vec<String>>(ids))
        .transpose()?;

    let start_date = data.start_date.unwrap_or(Utc::now() - Duration::weeks(2));
    let end_date = data.end_date.unwrap_or(Utc::now());
    let resolution_minutes = data
        .resolution_minutes
        .map_or(60 * 24, |minutes| minutes.get());

    // Convert String list to list of ProjectIds or VersionIds
    // - Filter out unauthorized projects/versions
    // - If no project_ids or version_ids are provided, we default to all projects the user has access to
    let project_ids =
        filter_allowed_ids(project_ids, user, &pool, &redis, None).await?;

    // Get the views
    let playtimes = crate::clickhouse::fetch_playtimes(
        project_ids.unwrap_or_default(),
        start_date,
        end_date,
        resolution_minutes,
        clickhouse.into_inner(),
    )
    .await?;

    let mut hm = HashMap::new();
    for playtime in playtimes {
        let id_string = to_base62(playtime.id);
        if !hm.contains_key(&id_string) {
            hm.insert(id_string.clone(), HashMap::new());
        }
        if let Some(hm) = hm.get_mut(&id_string) {
            hm.insert(playtime.time, playtime.total);
        }
    }

    Ok(HttpResponse::Ok().json(hm))
}

/// Get view data for a set of projects or versions
/// Data is returned as a hashmap of project/version ids to a hashmap of days to views
/// eg:
/// {
///     "4N1tEhnO": {
///         "20230824": 1090
///    }
///}
/// Either a list of project_ids or version_ids can be used, but not both. Unauthorized projects/versions will be filtered out.
pub async fn views_get(
    req: HttpRequest,
    clickhouse: web::Data<clickhouse::Client>,
    data: web::Query<GetData>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::ANALYTICS]),
    )
    .await
    .map(|x| x.1)?;

    let project_ids = data
        .project_ids
        .as_ref()
        .map(|ids| serde_json::from_str::<Vec<String>>(ids))
        .transpose()?;

    let start_date = data.start_date.unwrap_or(Utc::now() - Duration::weeks(2));
    let end_date = data.end_date.unwrap_or(Utc::now());
    let resolution_minutes = data
        .resolution_minutes
        .map_or(60 * 24, |minutes| minutes.get());

    // Convert String list to list of ProjectIds or VersionIds
    // - Filter out unauthorized projects/versions
    // - If no project_ids or version_ids are provided, we default to all projects the user has access to
    let project_ids =
        filter_allowed_ids(project_ids, user, &pool, &redis, None).await?;

    // Get the views
    let views = crate::clickhouse::fetch_views(
        project_ids.unwrap_or_default(),
        start_date,
        end_date,
        resolution_minutes,
        clickhouse.into_inner(),
    )
    .await?;

    let mut hm = HashMap::new();
    for views in views {
        let id_string = to_base62(views.id);
        if !hm.contains_key(&id_string) {
            hm.insert(id_string.clone(), HashMap::new());
        }
        if let Some(hm) = hm.get_mut(&id_string) {
            hm.insert(views.time, views.total);
        }
    }

    Ok(HttpResponse::Ok().json(hm))
}

/// Get download data for a set of projects or versions
/// Data is returned as a hashmap of project/version ids to a hashmap of days to downloads
/// eg:
/// {
///     "4N1tEhnO": {
///         "20230824": 32
///    }
///}
/// Either a list of project_ids or version_ids can be used, but not both. Unauthorized projects/versions will be filtered out.
pub async fn downloads_get(
    req: HttpRequest,
    clickhouse: web::Data<clickhouse::Client>,
    data: web::Query<GetData>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::ANALYTICS]),
    )
    .await
    .map(|x| x.1)?;

    let project_ids = data
        .project_ids
        .as_ref()
        .map(|ids| serde_json::from_str::<Vec<String>>(ids))
        .transpose()?;

    let start_date = data.start_date.unwrap_or(Utc::now() - Duration::weeks(2));
    let end_date = data.end_date.unwrap_or(Utc::now());
    let resolution_minutes = data
        .resolution_minutes
        .map_or(60 * 24, |minutes| minutes.get());

    // Convert String list to list of ProjectIds or VersionIds
    // - Filter out unauthorized projects/versions
    // - If no project_ids or version_ids are provided, we default to all projects the user has access to
    let project_ids =
        filter_allowed_ids(project_ids, user_option, &pool, &redis, None)
            .await?;

    // Get the downloads
    let downloads = crate::clickhouse::fetch_downloads(
        project_ids.unwrap_or_default(),
        start_date,
        end_date,
        resolution_minutes,
        clickhouse.into_inner(),
    )
    .await?;

    let mut hm = HashMap::new();
    for downloads in downloads {
        let id_string = to_base62(downloads.id);
        if !hm.contains_key(&id_string) {
            hm.insert(id_string.clone(), HashMap::new());
        }
        if let Some(hm) = hm.get_mut(&id_string) {
            hm.insert(downloads.time, downloads.total);
        }
    }

    Ok(HttpResponse::Ok().json(hm))
}

/// Get payout data for a set of projects
/// Data is returned as a hashmap of project ids to a hashmap of days to amount earned per day
/// eg:
/// {
///     "4N1tEhnO": {
///         "20230824": 0.001
///    }
///}
/// ONLY project IDs can be used. Unauthorized projects will be filtered out.
pub async fn revenue_get(
    req: HttpRequest,
    data: web::Query<GetData>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PAYOUTS_READ]),
    )
    .await
    .map(|x| x.1)?;

    let project_ids = data
        .project_ids
        .as_ref()
        .map(|ids| serde_json::from_str::<Vec<String>>(ids))
        .transpose()?;

    let start_date = data.start_date.unwrap_or(Utc::now() - Duration::weeks(2));
    let end_date = data.end_date.unwrap_or(Utc::now());
    let resolution_minutes = data
        .resolution_minutes
        .map_or(60 * 24, |minutes| minutes.get());

    // Round up/down to nearest duration as we are using pgadmin, does not have rounding in the fetch command
    // Round start_date down to nearest resolution
    let diff = start_date.timestamp() % (resolution_minutes as i64 * 60);
    let start_date = start_date - Duration::seconds(diff);

    // Round end_date up to nearest resolution
    let diff = end_date.timestamp() % (resolution_minutes as i64 * 60);
    let end_date =
        end_date + Duration::seconds((resolution_minutes as i64 * 60) - diff);

    // Convert String list to list of ProjectIds or VersionIds
    // - Filter out unauthorized projects/versions
    // - If no project_ids or version_ids are provided, we default to all projects the user has access to
    let project_ids = filter_allowed_ids(
        project_ids,
        user.clone(),
        &pool,
        &redis,
        Some(true),
    )
    .await?;

    let duration: PgInterval = Duration::minutes(resolution_minutes as i64)
        .try_into()
        .map_err(|_| {
            ApiError::InvalidInput("Invalid resolution_minutes".to_string())
        })?;
    // Get the revenue data
    let project_ids = project_ids.unwrap_or_default();

    struct PayoutValue {
        mod_id: Option<i64>,
        amount_sum: Option<rust_decimal::Decimal>,
        interval_start: Option<DateTime<Utc>>,
    }

    let payouts_values = if project_ids.is_empty() {
        sqlx::query!(
            "
            SELECT mod_id, SUM(amount) amount_sum, DATE_BIN($4::interval, created, TIMESTAMP '2001-01-01') AS interval_start
            FROM payouts_values
            WHERE user_id = $1 AND created BETWEEN $2 AND $3
            GROUP by mod_id, interval_start ORDER BY interval_start
            ",
            user.id.0 as i64,
            start_date,
            end_date,
            duration,
        )
            .fetch_all(&**pool)
            .await?.into_iter().map(|x| PayoutValue {
            mod_id: x.mod_id,
            amount_sum: x.amount_sum,
            interval_start: x.interval_start,
        }).collect::<Vec<_>>()
    } else {
        sqlx::query!(
            "
            SELECT mod_id, SUM(amount) amount_sum, DATE_BIN($4::interval, created, TIMESTAMP '2001-01-01') AS interval_start
            FROM payouts_values
            WHERE mod_id = ANY($1) AND created BETWEEN $2 AND $3
            GROUP by mod_id, interval_start ORDER BY interval_start
            ",
            &project_ids.iter().map(|x| x.0 as i64).collect::<Vec<_>>(),
            start_date,
            end_date,
            duration,
        )
            .fetch_all(&**pool)
            .await?.into_iter().map(|x| PayoutValue {
            mod_id: x.mod_id,
            amount_sum: x.amount_sum,
            interval_start: x.interval_start,
        }).collect::<Vec<_>>()
    };

    let mut hm: HashMap<_, _> = project_ids
        .into_iter()
        .map(|x| (x.to_string(), HashMap::new()))
        .collect::<HashMap<_, _>>();
    for value in payouts_values {
        if let Some(mod_id) = value.mod_id {
            if let Some(amount) = value.amount_sum {
                if let Some(interval_start) = value.interval_start {
                    let id_string = to_base62(mod_id as u64);
                    if !hm.contains_key(&id_string) {
                        hm.insert(id_string.clone(), HashMap::new());
                    }
                    if let Some(hm) = hm.get_mut(&id_string) {
                        hm.insert(interval_start.timestamp(), amount);
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(hm))
}

/// Get country data for a set of projects or versions
/// Data is returned as a hashmap of project/version ids to a hashmap of coutnry to downloads.
/// Unknown countries are labeled "".
/// This is usuable to see significant performing countries per project
/// eg:
/// {
///     "4N1tEhnO": {
///         "CAN":  22
///    }
///}
/// Either a list of project_ids or version_ids can be used, but not both. Unauthorized projects/versions will be filtered out.
/// For this endpoint, provided dates are a range to aggregate over, not specific days to fetch
pub async fn countries_downloads_get(
    req: HttpRequest,
    clickhouse: web::Data<clickhouse::Client>,
    data: web::Query<GetData>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::ANALYTICS]),
    )
    .await
    .map(|x| x.1)?;

    let project_ids = data
        .project_ids
        .as_ref()
        .map(|ids| serde_json::from_str::<Vec<String>>(ids))
        .transpose()?;

    let start_date = data.start_date.unwrap_or(Utc::now() - Duration::weeks(2));
    let end_date = data.end_date.unwrap_or(Utc::now());

    // Convert String list to list of ProjectIds or VersionIds
    // - Filter out unauthorized projects/versions
    // - If no project_ids or version_ids are provided, we default to all projects the user has access to
    let project_ids =
        filter_allowed_ids(project_ids, user, &pool, &redis, None).await?;

    // Get the countries
    let countries = crate::clickhouse::fetch_countries_downloads(
        project_ids.unwrap_or_default(),
        start_date,
        end_date,
        clickhouse.into_inner(),
    )
    .await?;

    let mut hm = HashMap::new();
    for views in countries {
        let id_string = to_base62(views.id);
        if !hm.contains_key(&id_string) {
            hm.insert(id_string.clone(), HashMap::new());
        }
        if let Some(hm) = hm.get_mut(&id_string) {
            hm.insert(views.country, views.total);
        }
    }

    let hm: HashMap<String, HashMap<String, u64>> = hm
        .into_iter()
        .map(|(key, value)| (key, condense_countries(value)))
        .collect();

    Ok(HttpResponse::Ok().json(hm))
}

/// Get country data for a set of projects or versions
/// Data is returned as a hashmap of project/version ids to a hashmap of coutnry to views.
/// Unknown countries are labeled "".
/// This is usuable to see significant performing countries per project
/// eg:
/// {
///     "4N1tEhnO": {
///         "CAN":  56165
///    }
///}
/// Either a list of project_ids or version_ids can be used, but not both. Unauthorized projects/versions will be filtered out.
/// For this endpoint, provided dates are a range to aggregate over, not specific days to fetch
pub async fn countries_views_get(
    req: HttpRequest,
    clickhouse: web::Data<clickhouse::Client>,
    data: web::Query<GetData>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::ANALYTICS]),
    )
    .await
    .map(|x| x.1)?;

    let project_ids = data
        .project_ids
        .as_ref()
        .map(|ids| serde_json::from_str::<Vec<String>>(ids))
        .transpose()?;

    let start_date = data.start_date.unwrap_or(Utc::now() - Duration::weeks(2));
    let end_date = data.end_date.unwrap_or(Utc::now());

    // Convert String list to list of ProjectIds or VersionIds
    // - Filter out unauthorized projects/versions
    // - If no project_ids or version_ids are provided, we default to all projects the user has access to
    let project_ids =
        filter_allowed_ids(project_ids, user, &pool, &redis, None).await?;

    // Get the countries
    let countries = crate::clickhouse::fetch_countries_views(
        project_ids.unwrap_or_default(),
        start_date,
        end_date,
        clickhouse.into_inner(),
    )
    .await?;

    let mut hm = HashMap::new();
    for views in countries {
        let id_string = to_base62(views.id);
        if !hm.contains_key(&id_string) {
            hm.insert(id_string.clone(), HashMap::new());
        }
        if let Some(hm) = hm.get_mut(&id_string) {
            hm.insert(views.country, views.total);
        }
    }

    let hm: HashMap<String, HashMap<String, u64>> = hm
        .into_iter()
        .map(|(key, value)| (key, condense_countries(value)))
        .collect();

    Ok(HttpResponse::Ok().json(hm))
}

fn condense_countries(countries: HashMap<String, u64>) -> HashMap<String, u64> {
    // Every country under '15' (view or downloads) should be condensed into 'XX'
    let mut hm = HashMap::new();
    for (mut country, count) in countries {
        if count < 50 {
            country = "XX".to_string();
        }
        if !hm.contains_key(&country) {
            hm.insert(country.to_string(), 0);
        }
        if let Some(hm) = hm.get_mut(&country) {
            *hm += count;
        }
    }
    hm
}

async fn filter_allowed_ids(
    mut project_ids: Option<Vec<String>>,
    user: crate::models::users::User,
    pool: &web::Data<PgPool>,
    redis: &RedisPool,
    remove_defaults: Option<bool>,
) -> Result<Option<Vec<ProjectId>>, ApiError> {
    // If no project_ids or version_ids are provided, we default to all projects the user has *public* access to
    if project_ids.is_none() && !remove_defaults.unwrap_or(false) {
        project_ids = Some(
            user_item::DBUser::get_projects(user.id.into(), &***pool, redis)
                .await?
                .into_iter()
                .map(|x| ProjectId::from(x).to_string())
                .collect(),
        );
    }

    // Convert String list to list of ProjectIds or VersionIds
    // - Filter out unauthorized projects/versions
    let project_ids = if let Some(project_strings) = project_ids {
        let projects_data = database::models::DBProject::get_many(
            &project_strings,
            &***pool,
            redis,
        )
        .await?;

        let team_ids = projects_data
            .iter()
            .map(|x| x.inner.team_id)
            .collect::<Vec<database::models::DBTeamId>>();
        let team_members =
            database::models::DBTeamMember::get_from_team_full_many(
                &team_ids, &***pool, redis,
            )
            .await?;

        let organization_ids = projects_data
            .iter()
            .filter_map(|x| x.inner.organization_id)
            .collect::<Vec<database::models::DBOrganizationId>>();
        let organizations = database::models::DBOrganization::get_many_ids(
            &organization_ids,
            &***pool,
            redis,
        )
        .await?;

        let organization_team_ids = organizations
            .iter()
            .map(|x| x.team_id)
            .collect::<Vec<database::models::DBTeamId>>();
        let organization_team_members =
            database::models::DBTeamMember::get_from_team_full_many(
                &organization_team_ids,
                &***pool,
                redis,
            )
            .await?;

        let ids = projects_data
            .into_iter()
            .filter(|project| {
                let team_member = team_members.iter().find(|x| {
                    x.team_id == project.inner.team_id
                        && x.user_id == user.id.into()
                });

                let organization = project
                    .inner
                    .organization_id
                    .and_then(|oid| organizations.iter().find(|x| x.id == oid));

                let organization_team_member =
                    if let Some(organization) = organization {
                        organization_team_members.iter().find(|x| {
                            x.team_id == organization.team_id
                                && x.user_id == user.id.into()
                        })
                    } else {
                        None
                    };

                let permissions = ProjectPermissions::get_permissions_by_role(
                    &user.role,
                    &team_member.cloned(),
                    &organization_team_member.cloned(),
                )
                .unwrap_or_default();

                permissions.contains(ProjectPermissions::VIEW_ANALYTICS)
            })
            .map(|x| x.inner.id.into())
            .collect::<Vec<_>>();

        Some(ids)
    } else {
        None
    };
    // Only one of project_ids or version_ids will be Some
    Ok(project_ids)
}
