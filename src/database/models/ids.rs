use super::DatabaseError;
use crate::models::ids::base62_impl::to_base62;
use crate::models::ids::random_base62_rng;
use censor::Censor;
use serde::Deserialize;
use sqlx::sqlx_macros::Type;

const ID_RETRY_COUNT: usize = 20;

macro_rules! generate_ids {
    ($vis:vis $function_name:ident, $return_type:ty, $id_length:expr, $select_stmnt:literal, $id_function:expr) => {
        $vis async fn $function_name(
            con: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ) -> Result<$return_type, DatabaseError> {
            let mut rng = rand::thread_rng();
            let length = $id_length;
            let mut id = random_base62_rng(&mut rng, length);
            let mut retry_count = 0;
            let censor = Censor::Standard + Censor::Sex;

            // Check if ID is unique
            loop {
                let results = sqlx::query!($select_stmnt, id as i64)
                    .fetch_one(&mut *con)
                    .await?;

                if results.exists.unwrap_or(true) || censor.check(&*to_base62(id)) {
                    id = random_base62_rng(&mut rng, length);
                } else {
                    break;
                }

                retry_count += 1;
                if retry_count > ID_RETRY_COUNT {
                    return Err(DatabaseError::RandomId);
                }
            }

            Ok($id_function(id as i64))
        }
    };
}

generate_ids!(
    pub generate_project_id,
    ProjectId,
    8,
    "SELECT EXISTS(SELECT 1 FROM mods WHERE id=$1)",
    ProjectId
);
generate_ids!(
    pub generate_version_id,
    VersionId,
    8,
    "SELECT EXISTS(SELECT 1 FROM versions WHERE id=$1)",
    VersionId
);
generate_ids!(
    pub generate_team_id,
    TeamId,
    8,
    "SELECT EXISTS(SELECT 1 FROM teams WHERE id=$1)",
    TeamId
);
generate_ids!(
    pub generate_file_id,
    FileId,
    8,
    "SELECT EXISTS(SELECT 1 FROM files WHERE id=$1)",
    FileId
);
generate_ids!(
    pub generate_team_member_id,
    TeamMemberId,
    8,
    "SELECT EXISTS(SELECT 1 FROM team_members WHERE id=$1)",
    TeamMemberId
);
generate_ids!(
    pub generate_state_id,
    StateId,
    8,
    "SELECT EXISTS(SELECT 1 FROM states WHERE id=$1)",
    StateId
);
generate_ids!(
    pub generate_user_id,
    UserId,
    8,
    "SELECT EXISTS(SELECT 1 FROM users WHERE id=$1)",
    UserId
);
generate_ids!(
    pub generate_report_id,
    ReportId,
    8,
    "SELECT EXISTS(SELECT 1 FROM reports WHERE id=$1)",
    ReportId
);

generate_ids!(
    pub generate_notification_id,
    NotificationId,
    8,
    "SELECT EXISTS(SELECT 1 FROM notifications WHERE id=$1)",
    NotificationId
);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Type)]
#[sqlx(transparent)]
pub struct UserId(pub i64);

#[derive(Copy, Clone, Debug, Type, Eq, PartialEq)]
#[sqlx(transparent)]
pub struct TeamId(pub i64);
#[derive(Copy, Clone, Debug, Type)]
#[sqlx(transparent)]
pub struct TeamMemberId(pub i64);

#[derive(Copy, Clone, Debug, Type, PartialEq, Eq, Deserialize, Hash)]
#[sqlx(transparent)]
pub struct ProjectId(pub i64);
#[derive(Copy, Clone, Debug, Type)]
#[sqlx(transparent)]
pub struct ProjectTypeId(pub i32);

#[derive(Copy, Clone, Debug, Type)]
#[sqlx(transparent)]
pub struct StatusId(pub i32);
#[derive(Copy, Clone, Debug, Type)]
#[sqlx(transparent)]
pub struct SideTypeId(pub i32);
#[derive(Copy, Clone, Debug, Type, Deserialize)]
#[sqlx(transparent)]
pub struct DonationPlatformId(pub i32);

#[derive(Copy, Clone, Debug, Type, PartialEq, Eq, Hash, Deserialize)]
#[sqlx(transparent)]
pub struct VersionId(pub i64);
#[derive(Copy, Clone, Debug, Type, Deserialize)]
#[sqlx(transparent)]
pub struct GameVersionId(pub i32);
#[derive(Copy, Clone, Debug, Type)]
#[sqlx(transparent)]
pub struct LoaderId(pub i32);
#[derive(Copy, Clone, Debug, Type)]
#[sqlx(transparent)]
pub struct CategoryId(pub i32);

#[derive(Copy, Clone, Debug, Type)]
#[sqlx(transparent)]
pub struct ReportId(pub i64);
#[derive(Copy, Clone, Debug, Type)]
#[sqlx(transparent)]
pub struct ReportTypeId(pub i32);

#[derive(Copy, Clone, Debug, Type, Hash, Eq, PartialEq, Deserialize)]
#[sqlx(transparent)]
pub struct FileId(pub i64);

#[derive(Copy, Clone, Debug, Type)]
#[sqlx(transparent)]
pub struct StateId(pub i64);

#[derive(Copy, Clone, Debug, Type, Deserialize)]
#[sqlx(transparent)]
pub struct NotificationId(pub i64);
#[derive(Copy, Clone, Debug, Type, Deserialize)]
#[sqlx(transparent)]
pub struct NotificationActionId(pub i32);

use crate::models::ids;

impl From<ids::ProjectId> for ProjectId {
    fn from(id: ids::ProjectId) -> Self {
        ProjectId(id.0 as i64)
    }
}
impl From<ProjectId> for ids::ProjectId {
    fn from(id: ProjectId) -> Self {
        ids::ProjectId(id.0 as u64)
    }
}
impl From<ids::UserId> for UserId {
    fn from(id: ids::UserId) -> Self {
        UserId(id.0 as i64)
    }
}
impl From<UserId> for ids::UserId {
    fn from(id: UserId) -> Self {
        ids::UserId(id.0 as u64)
    }
}
impl From<ids::TeamId> for TeamId {
    fn from(id: ids::TeamId) -> Self {
        TeamId(id.0 as i64)
    }
}
impl From<TeamId> for ids::TeamId {
    fn from(id: TeamId) -> Self {
        ids::TeamId(id.0 as u64)
    }
}
impl From<ids::VersionId> for VersionId {
    fn from(id: ids::VersionId) -> Self {
        VersionId(id.0 as i64)
    }
}
impl From<VersionId> for ids::VersionId {
    fn from(id: VersionId) -> Self {
        ids::VersionId(id.0 as u64)
    }
}
impl From<ids::ReportId> for ReportId {
    fn from(id: ids::ReportId) -> Self {
        ReportId(id.0 as i64)
    }
}
impl From<ReportId> for ids::ReportId {
    fn from(id: ReportId) -> Self {
        ids::ReportId(id.0 as u64)
    }
}
impl From<ids::NotificationId> for NotificationId {
    fn from(id: ids::NotificationId) -> Self {
        NotificationId(id.0 as i64)
    }
}
impl From<NotificationId> for ids::NotificationId {
    fn from(id: NotificationId) -> Self {
        ids::NotificationId(id.0 as u64)
    }
}
