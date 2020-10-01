use super::ids::Base62Id;
use crate::models::users::UserId;
use serde::{Deserialize, Serialize};

/// The ID of a team
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct TeamId(pub u64);

// TODO: permissions, role names, etc
/// A team of users who control a mod
#[derive(Serialize, Deserialize)]
pub struct Team {
    /// The id of the team
    pub id: TeamId,
    /// A list of the members of the team
    pub members: Vec<TeamMember>,
}

/// A member of a team
#[derive(Serialize, Deserialize, Clone)]
pub struct TeamMember {
    /// The ID of the user associated with the member
    pub user_id: UserId,
    /// The name of the user
    pub name: String,
    /// The role of the user in the team
    pub role: String,
}
