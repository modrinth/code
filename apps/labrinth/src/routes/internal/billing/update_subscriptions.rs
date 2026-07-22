use std::collections::HashMap;

use actix_web::{post, web};
use ariadne::ids::UserId as Id;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database::PgPool;
use crate::database::models::user_subscription_item::DBUserSubscription;
use crate::models::billing::SubscriptionMetadata;
use crate::routes::ApiError;
use crate::util::error::Context;
use crate::util::guards::subscriptions_key_guard;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateManySubscriptions {
    pub subscriptions: Vec<SubscriptionUpdate>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SubscriptionUpdate {
    pub target: SubscriptionTarget,
    pub update_region: Option<String>,
    pub ignore_if_missing: bool,
}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, ToSchema,
)]
pub enum SubscriptionTarget {
    Pyro { user_id: Id, server_id: Uuid },
}

/// Update multiple managed subscriptions.
#[utoipa::path(
	context_path = "/billing",
	tag = "billing",
	request_body = UpdateManySubscriptions,
	responses((status = OK))
)]
#[post(
    "/subscriptions/manager/update_many",
    guard = "subscriptions_key_guard"
)]
pub async fn update_many(
    pool: web::Data<PgPool>,
    body: web::Json<UpdateManySubscriptions>,
) -> Result<(), ApiError> {
    let UpdateManySubscriptions { subscriptions } = body.into_inner();

    let mut txn = pool
        .begin()
        .await
        .wrap_internal_err("failed to begin transaction")?;

    // Only supports hosting subscriptions now, so gather the server IDs and
    // fetch the subscriptions from them
    let server_ids = subscriptions
        .iter()
        .map(|update| match update.target {
            SubscriptionTarget::Pyro { server_id, .. } => server_id.to_string(),
        })
        .collect::<Vec<_>>();

    let found_subscriptions =
        DBUserSubscription::get_many_by_server_ids(&server_ids, &mut txn)
            .await
            .wrap_internal_err("failed to fetch subscriptions to update")?;

    let mut subscriptions_by_target = subscriptions
        .iter()
        .map(|update| (update.target, None))
        .collect::<HashMap<SubscriptionTarget, Option<DBUserSubscription>>>();

    // Creates a map of subscription "key" -> it's DB representation.
    for subscription in found_subscriptions {
        let Some(SubscriptionMetadata::Pyro { id, .. }) =
            subscription.metadata.as_ref()
        else {
            continue;
        };
        let Ok(server_id) = Uuid::parse_str(id) else {
            continue;
        };
        let target = SubscriptionTarget::Pyro {
            user_id: subscription.user_id.into(),
            server_id,
        };

        if let Some(entry) = subscriptions_by_target.get_mut(&target) {
            *entry = Some(subscription);
        }
    }

    for update in subscriptions {
        let Some(Some(subscription)) =
            subscriptions_by_target.get_mut(&update.target)
        else {
            continue;
        };

        // Update the subscription region
        if let Some(m) = subscription.metadata.as_mut() {
            if let SubscriptionMetadata::Pyro { region, .. } = m {
                if let Some(new_region) = update.update_region.clone() {
                    *region = Some(new_region);
                }
            }
        }

        subscription.upsert(&mut txn).await?;
    }

    txn.commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(())
}
