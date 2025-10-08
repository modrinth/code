use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::ids::AffiliateCodeId;

/// Affiliate code used to track referral purchases.
///
/// See [`AffiliateCode`].
///
/// This struct contains information which should only be visible to admins.
#[derive(Serialize, Deserialize)]
pub struct AdminAffiliateCode {
    /// Affiliate code ID.
    pub id: AffiliateCodeId,
    /// When the code was created.
    pub created_at: DateTime<Utc>,
    /// User who created the code.
    pub created_by: UserId,
    /// User who refers the purchaser.
    pub affiliate: UserId,
    /// Affiliate-defined name for this affiliate code - where the click came
    /// from.
    pub source_name: String,
}

impl From<crate::database::models::affiliate_code_item::DBAffiliateCode>
    for AdminAffiliateCode
{
    fn from(
        data: crate::database::models::affiliate_code_item::DBAffiliateCode,
    ) -> Self {
        Self {
            id: data.id.into(),
            created_at: data.created_at,
            created_by: data.created_by.into(),
            affiliate: data.affiliate.into(),
            source_name: data.source_name,
        }
    }
}

/// Affiliate code used to track referral purchases.
///
/// When a user follows a URL with [`AffiliateCode::id`] as an affiliate
/// parameter, the code will be saved as a cookie. When the same user purchases
/// a product with an affiliate code cookie, the purchase under that code is
/// tracked.
///
/// This struct contains information which is allowed to be seen by an
/// affiliate.
#[derive(Serialize, Deserialize)]
pub struct AffiliateCode {
    /// Affiliate code ID.
    pub id: AffiliateCodeId,
    /// User who refers the purchaser.
    pub affiliate: UserId,
    /// Affiliate-defined name for this affiliate code - where the click came
    /// from.
    pub source_name: String,
}

impl From<crate::database::models::affiliate_code_item::DBAffiliateCode>
    for AffiliateCode
{
    fn from(
        data: crate::database::models::affiliate_code_item::DBAffiliateCode,
    ) -> Self {
        Self {
            id: data.id.into(),
            affiliate: data.affiliate.into(),
            source_name: data.source_name,
        }
    }
}
