use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{
    DBUserId, ModerationFlagId, ModerationFlagRuleId, ModerationFlagRunId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBModerationFlagRule {
    pub id: ModerationFlagRuleId,
    pub name: String,
    pub created_by: Option<DBUserId>,
    pub created_at: DateTime<Utc>,
    pub updated_by: Option<DBUserId>,
    pub updated_at: DateTime<Utc>,
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBModerationFlag {
    pub id: ModerationFlagId,
    pub rule_id: Option<ModerationFlagRuleId>,
    pub what: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBModerationFlagRun {
    pub id: ModerationFlagRunId,
    pub status: FlagRunStatus,
    pub started_by: Option<DBUserId>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: Option<FlagRunOutput>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum FlagRunStatus {
    Running,
    Ok,
    Err,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagRunOutput {}
