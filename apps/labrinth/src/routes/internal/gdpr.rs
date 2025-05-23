use crate::auth::get_user_from_headers;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use actix_web::{HttpRequest, HttpResponse, post, web};
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("gdpr").service(export));
}

#[post("/export")]
pub async fn export(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let user_id = user.id.into();

    let collection_ids =
        crate::database::models::DBUser::get_collections(user_id, &**pool)
            .await?;
    let collections = crate::database::models::DBCollection::get_many(
        &collection_ids,
        &**pool,
        &redis,
    )
    .await?
    .into_iter()
    .map(crate::models::collections::Collection::from)
    .collect::<Vec<_>>();

    let follows =
        crate::database::models::DBUser::get_follows(user_id, &**pool)
            .await?
            .into_iter()
            .map(crate::models::ids::ProjectId::from)
            .collect::<Vec<_>>();

    let projects =
        crate::database::models::DBUser::get_projects(user_id, &**pool, &redis)
            .await?
            .into_iter()
            .map(crate::models::ids::ProjectId::from)
            .collect::<Vec<_>>();

    let org_ids =
        crate::database::models::DBUser::get_organizations(user_id, &**pool)
            .await?;
    let orgs =
        crate::database::models::organization_item::DBOrganization::get_many_ids(
            &org_ids, &**pool, &redis,
        )
        .await?
        .into_iter()
        // TODO: add team members
        .map(|x| crate::models::organizations::Organization::from(x, vec![]))
        .collect::<Vec<_>>();

    let notifs = crate::database::models::notification_item::DBNotification::get_many_user(
        user_id, &**pool, &redis,
    )
    .await?
    .into_iter()
    .map(crate::models::notifications::Notification::from)
    .collect::<Vec<_>>();

    let oauth_clients =
        crate::database::models::oauth_client_item::DBOAuthClient::get_all_user_clients(
            user_id, &**pool,
        )
        .await?
        .into_iter()
        .map(crate::models::oauth_clients::OAuthClient::from)
        .collect::<Vec<_>>();

    let oauth_authorizations = crate::database::models::oauth_client_authorization_item::DBOAuthClientAuthorization::get_all_for_user(
        user_id, &**pool,
    )
        .await?
        .into_iter()
        .map(crate::models::oauth_clients::OAuthClientAuthorization::from)
        .collect::<Vec<_>>();

    let pat_ids =
        crate::database::models::pat_item::DBPersonalAccessToken::get_user_pats(
            user_id, &**pool, &redis,
        )
        .await?;
    let pats =
        crate::database::models::pat_item::DBPersonalAccessToken::get_many_ids(
            &pat_ids, &**pool, &redis,
        )
        .await?
        .into_iter()
        .map(|x| crate::models::pats::PersonalAccessToken::from(x, false))
        .collect::<Vec<_>>();

    let payout_ids =
        crate::database::models::payout_item::DBPayout::get_all_for_user(
            user_id, &**pool,
        )
        .await?;

    let payouts = crate::database::models::payout_item::DBPayout::get_many(
        &payout_ids,
        &**pool,
    )
    .await?
    .into_iter()
    .map(crate::models::payouts::Payout::from)
    .collect::<Vec<_>>();

    let report_ids = crate::database::models::user_item::DBUser::get_reports(
        user_id, &**pool,
    )
    .await?;
    let reports = crate::database::models::report_item::DBReport::get_many(
        &report_ids,
        &**pool,
    )
    .await?
    .into_iter()
    .map(crate::models::reports::Report::from)
    .collect::<Vec<_>>();

    let message_ids = sqlx::query!(
        "
        SELECT id FROM threads_messages WHERE author_id = $1 AND hide_identity = FALSE
        ",
        user_id.0
    )
    .fetch_all(pool.as_ref())
    .await?
    .into_iter()
    .map(|x| crate::database::models::ids::DBThreadMessageId(x.id))
    .collect::<Vec<_>>();

    let messages =
        crate::database::models::thread_item::DBThreadMessage::get_many(
            &message_ids,
            &**pool,
        )
        .await?
        .into_iter()
        .map(|x| crate::models::threads::ThreadMessage::from(x, &user))
        .collect::<Vec<_>>();

    let uploaded_images_ids = sqlx::query!(
        "SELECT id FROM uploaded_images WHERE owner_id = $1",
        user_id.0
    )
    .fetch_all(pool.as_ref())
    .await?
    .into_iter()
    .map(|x| crate::database::models::ids::DBImageId(x.id))
    .collect::<Vec<_>>();

    let uploaded_images =
        crate::database::models::image_item::DBImage::get_many(
            &uploaded_images_ids,
            &**pool,
            &redis,
        )
        .await?
        .into_iter()
        .map(crate::models::images::Image::from)
        .collect::<Vec<_>>();

    let subscriptions =
        crate::database::models::user_subscription_item::DBUserSubscription::get_all_user(
            user_id, &**pool,
        )
        .await?
        .into_iter()
        .map(crate::models::billing::UserSubscription::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user": user,
        "collections": collections,
        "follows": follows,
        "projects": projects,
        "orgs": orgs,
        "notifs": notifs,
        "oauth_clients": oauth_clients,
        "oauth_authorizations": oauth_authorizations,
        "pats": pats,
        "payouts": payouts,
        "reports": reports,
        "messages": messages,
        "uploaded_images": uploaded_images,
        "subscriptions": subscriptions,
    })))
}
