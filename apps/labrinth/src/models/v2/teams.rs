use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::{
    ids::TeamId,
    teams::{ProjectPermissions, TeamMember},
    users::User,
};

/// A member of a team
#[derive(Serialize, Deserialize, Clone)]
pub struct LegacyTeamMember {
    pub role: String,
    // is_owner removed, and role hardcoded to Owner if true,
    pub team_id: TeamId,
    pub user: User,
    pub permissions: Option<ProjectPermissions>,
    pub accepted: bool,

    #[serde(with = "rust_decimal::serde::float_option")]
    pub payouts_split: Option<Decimal>,
    pub ordering: i64,
}

impl LegacyTeamMember {
    pub fn from(team_member: TeamMember) -> Self {
        LegacyTeamMember {
            role: match (team_member.is_owner, team_member.role.as_str()) {
                (true, _) => "Owner".to_string(),
                (false, "Owner") => "Member".to_string(), // The odd case of a non-owner with the owner role should show as 'Member'
                (false, role) => role.to_string(),
            },
            team_id: team_member.team_id,
            user: team_member.user,
            permissions: team_member.permissions,
            accepted: team_member.accepted,
            payouts_split: team_member.payouts_split,
            ordering: team_member.ordering,
        }
    }
}
