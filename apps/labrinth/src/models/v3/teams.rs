use crate::bitflags_serde_impl;
use crate::models::ids::TeamId;
use crate::models::users::User;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub const DEFAULT_ROLE: &str = "Member";

/// A team of users who control a project
#[derive(Serialize, Deserialize)]
pub struct Team {
    /// The id of the team
    pub id: TeamId,
    /// A list of the members of the team
    pub members: Vec<TeamMember>,
}

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct ProjectPermissions: u64 {
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
    }
}

bitflags_serde_impl!(ProjectPermissions, u64);

impl Default for ProjectPermissions {
    fn default() -> ProjectPermissions {
        ProjectPermissions::empty()
    }
}

impl ProjectPermissions {
    pub fn get_permissions_by_role(
        role: &crate::models::users::Role,
        project_team_member: &Option<crate::database::models::DBTeamMember>, // team member of the user in the project
        organization_team_member: &Option<
            crate::database::models::DBTeamMember,
        >, // team member of the user in the organization
    ) -> Option<Self> {
        if role.is_admin() {
            return Some(ProjectPermissions::all());
        }

        if let Some(member) = project_team_member {
            if member.accepted {
                return Some(member.permissions);
            }
        }

        if let Some(member) = organization_team_member {
            if member.accepted {
                return Some(member.permissions);
            }
        }

        if role.is_mod() {
            Some(
                ProjectPermissions::EDIT_DETAILS
                    | ProjectPermissions::EDIT_BODY
                    | ProjectPermissions::UPLOAD_VERSION,
            )
        } else {
            None
        }
    }
}

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct OrganizationPermissions: u64 {
        const EDIT_DETAILS = 1 << 0;
        const MANAGE_INVITES = 1 << 1;
        const REMOVE_MEMBER = 1 << 2;
        const EDIT_MEMBER = 1 << 3;
        const ADD_PROJECT = 1 << 4;
        const REMOVE_PROJECT = 1 << 5;
        const DELETE_ORGANIZATION = 1 << 6;
        const EDIT_MEMBER_DEFAULT_PERMISSIONS = 1 << 7; // Separate from EDIT_MEMBER
        const NONE = 0b0;
    }
}

bitflags_serde_impl!(OrganizationPermissions, u64);

impl Default for OrganizationPermissions {
    fn default() -> OrganizationPermissions {
        OrganizationPermissions::NONE
    }
}

impl OrganizationPermissions {
    pub fn get_permissions_by_role(
        role: &crate::models::users::Role,
        team_member: &Option<crate::database::models::DBTeamMember>,
    ) -> Option<Self> {
        if role.is_admin() {
            return Some(OrganizationPermissions::all());
        }

        if let Some(member) = team_member {
            if member.accepted {
                return member.organization_permissions;
            }
        }
        if role.is_mod() {
            return Some(
                OrganizationPermissions::EDIT_DETAILS
                    | OrganizationPermissions::ADD_PROJECT,
            );
        }
        None
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
    /// Is the user the owner of the team?
    pub is_owner: bool,
    /// A bitset containing the user's permissions in this team.
    /// In an organization-controlled project, these are the unique overriding permissions for the user's role for any project in the organization, if they exist.
    /// In an organization, these are the default project permissions for any project in the organization.
    /// Not optional- only None if they are being hidden from the user.
    pub permissions: Option<ProjectPermissions>,

    /// A bitset containing the user's permissions in this organization.
    /// In a project team, this is None.
    pub organization_permissions: Option<OrganizationPermissions>,

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
    pub fn from(
        data: crate::database::models::team_item::DBTeamMember,
        user: crate::database::models::DBUser,
        override_permissions: bool,
    ) -> Self {
        let user: User = user.into();
        Self::from_model(data, user, override_permissions)
    }

    // Use the User model directly instead of the database model,
    // if already available.
    // (Avoids a db query in some cases)
    pub fn from_model(
        data: crate::database::models::team_item::DBTeamMember,
        user: crate::models::users::User,
        override_permissions: bool,
    ) -> Self {
        Self {
            team_id: data.team_id.into(),
            user,
            role: data.role,
            is_owner: data.is_owner,
            permissions: if override_permissions {
                None
            } else {
                Some(data.permissions)
            },
            organization_permissions: if override_permissions {
                None
            } else {
                data.organization_permissions
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
