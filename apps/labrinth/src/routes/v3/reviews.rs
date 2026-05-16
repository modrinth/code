use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::models::review_item::DBReview;
use crate::database::redis::RedisPool;
use crate::models::ids::{ProjectId, ReviewId};
use crate::models::pats::Scopes;
use crate::models::v3::reviews::Review;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::routes::read_typed_from_payload;
use actix_web::{HttpRequest, HttpResponse, web};
use ariadne::ids::base62_impl::parse_base62;
use chrono::Utc;
use serde::Deserialize;
use validator::Validate;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("review", web::post().to(review_create));
    cfg.route("reviews", web::get().to(reviews_get));
    cfg.route("review/{id}", web::get().to(review_get));
    cfg.route("review/{id}", web::patch().to(review_edit));
    cfg.route("review/{id}", web::delete().to(review_delete));
    cfg.route("review/project/{id}", web::get().to(project_review_get));
}

#[derive(Deserialize, Validate)]
pub struct CreateReview {
    pub project_id: String,
    #[validate(range(min = 1, max = 5))]
    pub rating: i16,
    #[validate(length(max = 8192))]
    #[serde(default)]
    pub body: String,
}

pub async fn review_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    mut body: web::Payload,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::REVIEW_CREATE,
    )
    .await?
    .1;

    let new_review: CreateReview = read_typed_from_payload(&mut body).await?;
    new_review.validate().map_err(|e| {
        ApiError::InvalidInput(format!("Validation error: {e}"))
    })?;

    let project_id =
        ProjectId(parse_base62(new_review.project_id.as_str())?);

    let project_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM mods WHERE id = $1)",
        project_id.0 as i64
    )
    .fetch_one(&**pool)
    .await?
    .exists
    .unwrap_or(false);

    if !project_exists {
        return Err(ApiError::InvalidInput(format!(
            "Project could not be found: {}",
            new_review.project_id
        )));
    }

    let db_project_id: crate::database::models::ids::DBProjectId =
        project_id.into();
    let db_user_id: crate::database::models::ids::DBUserId =
        current_user.id.into();

    let existing = DBReview::get_by_user_and_project(
        db_user_id,
        db_project_id,
        &**pool,
    )
    .await?;

    if existing.is_some() {
        return Err(ApiError::InvalidInput(
            "You have already reviewed this project. Edit or delete your existing review.".to_string(),
        ));
    }

    let mut transaction = pool.begin().await?;

    let id = crate::database::models::generate_review_id(&mut transaction).await?;
    let now = Utc::now();

    let review = DBReview {
        id,
        project_id: db_project_id,
        user_id: db_user_id,
        rating: new_review.rating,
        body: new_review.body.clone(),
        created: now,
        updated: now,
    };

    review.insert(&mut transaction).await?;
    transaction.commit().await?;

    Ok(HttpResponse::Ok().json(Review {
        id: id.into(),
        project_id,
        user_id: current_user.id,
        rating: new_review.rating,
        body: new_review.body,
        created: now,
        updated: now,
    }))
}

#[derive(Deserialize)]
pub struct ReviewsQuery {
    pub project_id: String,
    #[serde(default = "default_count")]
    pub count: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_count() -> i64 {
    20
}

pub async fn reviews_get(
    pool: web::Data<PgPool>,
    web::Query(query): web::Query<ReviewsQuery>,
) -> Result<HttpResponse, ApiError> {
    let project_id = ProjectId(parse_base62(query.project_id.as_str())?);
    let db_project_id: crate::database::models::ids::DBProjectId =
        project_id.into();

    let reviews = DBReview::get_for_project(
        db_project_id,
        &**pool,
        query.count.min(100),
        query.offset,
    )
    .await?;

    let total = DBReview::count_for_project(db_project_id, &**pool).await?;

    let reviews: Vec<Review> = reviews.into_iter().map(|r| r.into()).collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "reviews": reviews,
        "total": total,
    })))
}

pub async fn review_get(
    pool: web::Data<PgPool>,
    info: web::Path<(ReviewId,)>,
) -> Result<HttpResponse, ApiError> {
    let id: crate::database::models::ids::DBReviewId = info.into_inner().0.into();

    let review = DBReview::get(id, &**pool).await?;

    match review {
        Some(r) => Ok(HttpResponse::Ok().json(Review::from(r))),
        None => Err(ApiError::NotFound),
    }
}

pub async fn project_review_get(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    info: web::Path<(ProjectId,)>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::REVIEW_READ,
    )
    .await?
    .1;

    let project_id = info.into_inner().0;
    let db_project_id: crate::database::models::ids::DBProjectId =
        project_id.into();
    let db_user_id: crate::database::models::ids::DBUserId =
        current_user.id.into();

    let review =
        DBReview::get_by_user_and_project(db_user_id, db_project_id, &**pool)
            .await?;

    match review {
        Some(r) => Ok(HttpResponse::Ok().json(Review::from(r))),
        None => Err(ApiError::NotFound),
    }
}

#[derive(Deserialize, Validate)]
pub struct EditReview {
    #[validate(range(min = 1, max = 5))]
    pub rating: Option<i16>,
    #[validate(length(max = 8192))]
    pub body: Option<String>,
}

pub async fn review_edit(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    info: web::Path<(ReviewId,)>,
    session_queue: web::Data<AuthQueue>,
    edit_review: web::Json<EditReview>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::REVIEW_WRITE,
    )
    .await?
    .1;

    edit_review.validate().map_err(|e| {
        ApiError::InvalidInput(format!("Validation error: {e}"))
    })?;

    let id: crate::database::models::ids::DBReviewId =
        info.into_inner().0.into();
    let review = DBReview::get(id, &**pool).await?;

    let Some(review) = review else {
        return Err(ApiError::NotFound);
    };

    if review.user_id != current_user.id.into()
        && !current_user.role.is_mod()
    {
        return Err(ApiError::NotFound);
    }

    let mut transaction = pool.begin().await?;

    if let Some(rating) = edit_review.rating {
        sqlx::query!(
            "UPDATE project_reviews SET rating = $1, updated = NOW() WHERE id = $2",
            rating,
            id as crate::database::models::ids::DBReviewId,
        )
        .execute(&mut *transaction)
        .await?;
    }

    if let Some(ref body) = edit_review.body {
        sqlx::query!(
            "UPDATE project_reviews SET body = $1, updated = NOW() WHERE id = $2",
            body,
            id as crate::database::models::ids::DBReviewId,
        )
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn review_delete(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    info: web::Path<(ReviewId,)>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::REVIEW_DELETE,
    )
    .await?
    .1;

    let id: crate::database::models::ids::DBReviewId =
        info.into_inner().0.into();
    let review = DBReview::get(id, &**pool).await?;

    let Some(review) = review else {
        return Err(ApiError::NotFound);
    };

    if review.user_id != current_user.id.into()
        && !current_user.role.is_mod()
    {
        return Err(ApiError::NotFound);
    }

    let mut transaction = pool.begin().await?;
    DBReview::remove(id, &mut transaction).await?;
    transaction.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}
