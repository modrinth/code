use actix_web::{HttpRequest, http::header::IF_MATCH};
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::routes::ApiError;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ModerationNote {
    pub notes: String,
    pub last_modified: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_author: UserId,
    pub user_rating: i32,
    pub version: i32,
}

impl From<crate::database::models::DBModerationNote> for ModerationNote {
    fn from(note: crate::database::models::DBModerationNote) -> Self {
        Self {
            notes: note.notes,
            last_modified: note.last_modified,
            created_at: note.created_at,
            last_author: note.last_author.into(),
            user_rating: note.user_rating,
            version: note.version,
        }
    }
}

#[derive(Deserialize)]
pub struct PatchModerationNote {
    pub notes: Option<String>,
    pub user_rating: Option<i32>,
}

impl PatchModerationNote {
    pub fn validate_not_empty(&self) -> Result<(), ApiError> {
        if self.notes.is_none() && self.user_rating.is_none() {
            return Err(ApiError::InvalidInput(
                "must specify `notes` or `user_rating`".to_string(),
            ));
        }

        Ok(())
    }
}

pub fn parse_if_match_header(
    req: &HttpRequest,
) -> Result<Option<i32>, ApiError> {
    let Some(value) = req.headers().get(IF_MATCH) else {
        return Ok(None);
    };

    let value = value.to_str().map_err(|_| {
        ApiError::InvalidInput(
            "`if-match` header must be a valid integer".to_string(),
        )
    })?;

    Some(value.parse::<i32>().map_err(|_| {
        ApiError::InvalidInput(
            "`if-match` header must be a valid integer".to_string(),
        )
    }))
    .transpose()
}
