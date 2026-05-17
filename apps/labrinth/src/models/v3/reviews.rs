use crate::database::models::review_item::DBReview;
use crate::models::ids::{ProjectId, ReviewId};
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Review {
    pub id: ReviewId,
    pub project_id: ProjectId,
    pub user_id: UserId,
    pub rating: i16,
    pub body: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl From<DBReview> for Review {
    fn from(x: DBReview) -> Self {
        Review {
            id: x.id.into(),
            project_id: x.project_id.into(),
            user_id: x.user_id.into(),
            rating: x.rating,
            body: x.body,
            created: x.created,
            updated: x.updated,
        }
    }
}
