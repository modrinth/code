use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::ids::AffiliateCodeId;

/// Affiliate code used to track referral purchases.
///
/// When a user follows a URL with [`AffiliateCode::id`] as an affiliate
/// parameter, the code will be saved as a cookie. When the same user purchases
/// a product with an affiliate code cookie, the purchase under that code is
/// tracked.
///
/// This struct contains information which is allowed to be seen by an
/// affiliate.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AffiliateCode {
    /// Affiliate code ID.
    pub id: AffiliateCodeId,
    /// When the code was created.
    pub created_at: Option<DateTime<Utc>>,
    /// User who created the code.
    pub created_by: Option<UserId>,
    /// User who refers the purchaser.
    pub affiliate: UserId,
    /// Affiliate-defined name for this affiliate code - where the click came
    /// from.
    pub source_name: String,
}

impl AffiliateCode {
    pub fn from(
        data: crate::database::models::affiliate_code_item::DBAffiliateCode,
        is_admin: bool,
    ) -> Self {
        Self {
            id: data.id.into(),
            created_at: if is_admin {
                Some(data.created_at)
            } else {
                None
            },
            created_by: if is_admin {
                Some(data.created_by.into())
            } else {
                None
            },
            affiliate: data.affiliate.into(),
            source_name: data.source_name,
        }
    }
}
