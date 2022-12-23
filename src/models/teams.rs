use super::ids::Base62Id;
use crate::database::models::team_item::QueryTeamMember;
use crate::models::users::User;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The ID of a team
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct TeamId(pub u64);

pub const OWNER_ROLE: &str = "Owner";
pub const DEFAULT_ROLE: &str = "Member";

// TODO: permissions, role names, etc
/// A team of users who control a project
#[derive(Serialize, Deserialize)]
pub struct Team {
    /// The id of the team
    pub id: TeamId,
    /// A list of the members of the team
    pub members: Vec<TeamMember>,
}

bitflags::bitflags! {
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Permissions: u64 {
        const UPLOAD_VERSION = 1 << 0;
        const DELETE_VERSION = 1 << 1;
        const EDIT_DETAILS = 1 << 2;
        const EDIT_BODY = 1 << 3;
        const MANAGE_INVITES = 1 << 4;
        const REMOVE_MEMBER = 1 << 5;
        const EDIT_MEMBER = 1 << 6;
        const DELETE_PROJECT = 1 << 7;
        const VIEW_ANALYTICS = 1 << 8;
        const VIEW_PAYOUTS = 1 << 9;
        const ALL = 0b1111111111;
    }
}

impl Default for Permissions {
    fn default() -> Permissions {
        Permissions::UPLOAD_VERSION | Permissions::DELETE_VERSION
    }
}

/// A member of a team
#[derive(Serialize, Deserialize, Clone)]
pub struct TeamMember {
    /// The ID of the team this team member is a member of
    pub team_id: TeamId,
    /// The user associated with the member
    pub user: User,
    /// The role of the user in the team
    pub role: String,
    /// A bitset containing the user's permissions in this team
    pub permissions: Option<Permissions>,
    /// Whether the user has joined the team or is just invited to it
    pub accepted: bool,

    #[serde(with = "rust_decimal::serde::float_option")]
    /// Payouts split. This is a weighted average. For example. if a team has two members with this
    /// value set to 25.0 for both members, they split revenue 50/50
    pub payouts_split: Option<Decimal>,
    /// Ordering of the member in the list
    pub ordering: i64,
}

impl TeamMember {
    pub fn from(data: QueryTeamMember, override_permissions: bool) -> Self {
        let has_flame_anvil_key = data.user.flame_anvil_key.is_some();
        let mut user: User = data.user.into();

        if !override_permissions {
            user.has_flame_anvil_key = Some(has_flame_anvil_key);
        }

        Self {
            team_id: data.team_id.into(),
            user,
            role: data.role,
            permissions: if override_permissions {
                None
            } else {
                Some(data.permissions)
            },
            accepted: data.accepted,
            payouts_split: if override_permissions {
                None
            } else {
                Some(data.payouts_split)
            },
            ordering: data.ordering,
        }
    }
}
