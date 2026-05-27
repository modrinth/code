use actix_web::{post, web};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::routes::ApiError;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(tiltify_webhook);
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct TiltifyWebhook {
    pub data: TiltifyDonation,
    pub meta: TiltifyWebhookMeta,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct TiltifyDonation {
    pub amount: TiltifyAmount,
    pub campaign_id: Uuid,
    pub completed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub donor_comment: Option<String>,
    pub donor_name: String,
    pub fundraising_event_id: Option<Uuid>,
    pub id: Uuid,
    pub legacy_id: i64,
    pub poll_id: Option<Uuid>,
    pub poll_option_id: Option<Uuid>,
    pub reward_id: Option<Uuid>,
    pub sustained: bool,
    pub target_id: Option<Uuid>,
    pub team_event_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct TiltifyAmount {
    pub currency: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct TiltifyWebhookMeta {
    pub attempted_at: DateTime<Utc>,
    pub event_type: String,
    pub generated_at: DateTime<Utc>,
    pub id: Uuid,
    pub subscription_source_id: Uuid,
    pub subscription_source_type: String,
}

#[utoipa::path]
#[post("/webhook")]
pub async fn tiltify_webhook(
    _payload: web::Json<TiltifyWebhook>,
) -> Result<(), ApiError> {
    Ok(())
}
