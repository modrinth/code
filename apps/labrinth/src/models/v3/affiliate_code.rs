use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::ids::AffiliateCodeId;

#[derive(Serialize, Deserialize)]
pub struct AffiliateCode {
    pub id: AffiliateCodeId,
    pub created_at: Option<DateTime<Utc>>,
    pub created_by: Option<UserId>,
    pub affiliate: UserId,
}

impl AffiliateCode {
    pub fn from(
        data: crate::database::models::affiliate_code_item::DBAffiliateCode,
        admin_view: bool,
    ) -> Self {
        Self {
            id: data.id.into(),
            created_at: if admin_view {
                Some(data.created_at)
            } else {
                None
            },
            created_by: if admin_view {
                Some(data.created_by.into())
            } else {
                None
            },
            affiliate: data.affiliate.into(),
        }
    }
}
