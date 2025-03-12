use super::DatabaseError;
use ariadne::ids::base62_impl::to_base62;
use ariadne::ids::{random_base62_rng, random_base62_rng_range};
use censor::Censor;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sqlx::sqlx_macros::Type;

const ID_RETRY_COUNT: usize = 20;

macro_rules! generate_ids {
    ($vis:vis $function_name:ident, $return_type:ty, $id_length:expr, $select_stmnt:literal, $id_function:expr) => {
        $vis async fn $function_name(
            con: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ) -> Result<$return_type, DatabaseError> {
            let mut rng = ChaCha20Rng::from_entropy();
            let length = $id_length;
            let mut id = random_base62_rng(&mut rng, length);
            let mut retry_count = 0;
            let censor = Censor::Standard + Censor::Sex;

            // Check if ID is unique
            loop {
                let results = sqlx::query!($select_stmnt, id as i64)
                    .fetch_one(&mut **con)
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

macro_rules! generate_bulk_ids {
    ($vis:vis $function_name:ident, $return_type:ty, $select_stmnt:literal, $id_function:expr) => {
        $vis async fn $function_name(
            count: usize,
            con: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ) -> Result<Vec<$return_type>, DatabaseError> {
            let mut rng = rand::thread_rng();
            let mut retry_count = 0;

            // Check if ID is unique
            loop {
                let base = random_base62_rng_range(&mut rng, 1, 10) as i64;
                let ids = (0..count).map(|x| base + x as i64).collect::<Vec<_>>();

                let results = sqlx::query!($select_stmnt, &ids)
                    .fetch_one(&mut **con)
                    .await?;

                if !results.exists.unwrap_or(true) {
                    return Ok(ids.into_iter().map(|x| $id_function(x)).collect());
                }

                retry_count += 1;
                if retry_count > ID_RETRY_COUNT {
                    return Err(DatabaseError::RandomId);
                }
            }
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
    pub generate_organization_id,
    OrganizationId,
    8,
    "SELECT EXISTS(SELECT 1 FROM organizations WHERE id=$1)",
    OrganizationId
);
generate_ids!(
    pub generate_collection_id,
    CollectionId,
    8,
    "SELECT EXISTS(SELECT 1 FROM collections WHERE id=$1)",
    CollectionId
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
    pub generate_pat_id,
    PatId,
    8,
    "SELECT EXISTS(SELECT 1 FROM pats WHERE id=$1)",
    PatId
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

generate_bulk_ids!(
    pub generate_many_notification_ids,
    NotificationId,
    "SELECT EXISTS(SELECT 1 FROM notifications WHERE id = ANY($1))",
    NotificationId
);

generate_ids!(
    pub generate_thread_id,
    ThreadId,
    8,
    "SELECT EXISTS(SELECT 1 FROM threads WHERE id=$1)",
    ThreadId
);
generate_ids!(
    pub generate_thread_message_id,
    ThreadMessageId,
    8,
    "SELECT EXISTS(SELECT 1 FROM threads_messages WHERE id=$1)",
    ThreadMessageId
);

generate_ids!(
    pub generate_session_id,
    SessionId,
    8,
    "SELECT EXISTS(SELECT 1 FROM sessions WHERE id=$1)",
    SessionId
);

generate_ids!(
    pub generate_image_id,
    ImageId,
    8,
    "SELECT EXISTS(SELECT 1 FROM uploaded_images WHERE id=$1)",
    ImageId
);

generate_ids!(
    pub generate_oauth_client_authorization_id,
    OAuthClientAuthorizationId,
    8,
    "SELECT EXISTS(SELECT 1 FROM oauth_client_authorizations WHERE id=$1)",
    OAuthClientAuthorizationId
);

generate_ids!(
    pub generate_oauth_client_id,
    OAuthClientId,
    8,
    "SELECT EXISTS(SELECT 1 FROM oauth_clients WHERE id=$1)",
    OAuthClientId
);

generate_ids!(
    pub generate_oauth_redirect_id,
    OAuthRedirectUriId,
    8,
    "SELECT EXISTS(SELECT 1 FROM oauth_client_redirect_uris WHERE id=$1)",
    OAuthRedirectUriId
);

generate_ids!(
    pub generate_oauth_access_token_id,
    OAuthAccessTokenId,
    8,
    "SELECT EXISTS(SELECT 1 FROM oauth_access_tokens WHERE id=$1)",
    OAuthAccessTokenId
);

generate_ids!(
    pub generate_payout_id,
    PayoutId,
    8,
    "SELECT EXISTS(SELECT 1 FROM oauth_access_tokens WHERE id=$1)",
    PayoutId
);

generate_ids!(
    pub generate_product_id,
    ProductId,
    8,
    "SELECT EXISTS(SELECT 1 FROM products WHERE id=$1)",
    ProductId
);

generate_ids!(
    pub generate_product_price_id,
    ProductPriceId,
    8,
    "SELECT EXISTS(SELECT 1 FROM products_prices WHERE id=$1)",
    ProductPriceId
);

generate_ids!(
    pub generate_user_subscription_id,
    UserSubscriptionId,
    8,
    "SELECT EXISTS(SELECT 1 FROM users_subscriptions WHERE id=$1)",
    UserSubscriptionId
);

generate_ids!(
    pub generate_charge_id,
    ChargeId,
    8,
    "SELECT EXISTS(SELECT 1 FROM charges WHERE id=$1)",
    ChargeId
);

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Type, Hash, Serialize, Deserialize,
)]
#[sqlx(transparent)]
pub struct UserId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, Eq, Hash, PartialEq, Serialize, Deserialize,
)]
#[sqlx(transparent)]
pub struct TeamId(pub i64);
#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct TeamMemberId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[sqlx(transparent)]
pub struct OrganizationId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[sqlx(transparent)]
pub struct ProjectId(pub i64);
#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, PartialEq, Eq, Hash,
)]
#[sqlx(transparent)]
pub struct ProjectTypeId(pub i32);

#[derive(Copy, Clone, Debug, Type)]
#[sqlx(transparent)]
pub struct StatusId(pub i32);
#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct GameId(pub i32);
#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, PartialEq, Eq, Hash,
)]
#[sqlx(transparent)]
pub struct LinkPlatformId(pub i32);

#[derive(
    Copy,
    Clone,
    Debug,
    Type,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    PartialOrd,
    Ord,
)]
#[sqlx(transparent)]
pub struct VersionId(pub i64);
#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, PartialEq, Eq, Hash,
)]
#[sqlx(transparent)]
pub struct LoaderId(pub i32);
#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct CategoryId(pub i32);

#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct CollectionId(pub i64);

#[derive(Copy, Clone, Debug, Type, Deserialize, Serialize)]
#[sqlx(transparent)]
pub struct ReportId(pub i64);
#[derive(Copy, Clone, Debug, Type)]
#[sqlx(transparent)]
pub struct ReportTypeId(pub i32);

#[derive(
    Copy, Clone, Debug, Type, Hash, Eq, PartialEq, Deserialize, Serialize,
)]
#[sqlx(transparent)]
pub struct FileId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, Deserialize, Serialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct PatId(pub i64);

#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct NotificationId(pub i64);
#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct NotificationActionId(pub i32);

#[derive(Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq)]
#[sqlx(transparent)]
pub struct ThreadId(pub i64);
#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct ThreadMessageId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct SessionId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct ImageId(pub i64);

#[derive(
    Copy,
    Clone,
    Debug,
    Type,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Hash,
    PartialOrd,
    Ord,
)]
#[sqlx(transparent)]
pub struct LoaderFieldId(pub i32);

#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct LoaderFieldEnumId(pub i32);

#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct LoaderFieldEnumValueId(pub i32);

#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct OAuthClientId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct OAuthClientAuthorizationId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct OAuthRedirectUriId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct OAuthAccessTokenId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct PayoutId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct ProductId(pub i64);
#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct ProductPriceId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct UserSubscriptionId(pub i64);

#[derive(
    Copy, Clone, Debug, Type, Serialize, Deserialize, Eq, PartialEq, Hash,
)]
#[sqlx(transparent)]
pub struct ChargeId(pub i64);

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
impl From<ids::OrganizationId> for OrganizationId {
    fn from(id: ids::OrganizationId) -> Self {
        OrganizationId(id.0 as i64)
    }
}
impl From<OrganizationId> for ids::OrganizationId {
    fn from(id: OrganizationId) -> Self {
        ids::OrganizationId(id.0 as u64)
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
impl From<ids::CollectionId> for CollectionId {
    fn from(id: ids::CollectionId) -> Self {
        CollectionId(id.0 as i64)
    }
}
impl From<CollectionId> for ids::CollectionId {
    fn from(id: CollectionId) -> Self {
        ids::CollectionId(id.0 as u64)
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
impl From<ImageId> for ids::ImageId {
    fn from(id: ImageId) -> Self {
        ids::ImageId(id.0 as u64)
    }
}
impl From<ids::ImageId> for ImageId {
    fn from(id: ids::ImageId) -> Self {
        ImageId(id.0 as i64)
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
impl From<ids::ThreadId> for ThreadId {
    fn from(id: ids::ThreadId) -> Self {
        ThreadId(id.0 as i64)
    }
}
impl From<ThreadId> for ids::ThreadId {
    fn from(id: ThreadId) -> Self {
        ids::ThreadId(id.0 as u64)
    }
}
impl From<ids::ThreadMessageId> for ThreadMessageId {
    fn from(id: ids::ThreadMessageId) -> Self {
        ThreadMessageId(id.0 as i64)
    }
}
impl From<ThreadMessageId> for ids::ThreadMessageId {
    fn from(id: ThreadMessageId) -> Self {
        ids::ThreadMessageId(id.0 as u64)
    }
}
impl From<SessionId> for ids::SessionId {
    fn from(id: SessionId) -> Self {
        ids::SessionId(id.0 as u64)
    }
}
impl From<PatId> for ids::PatId {
    fn from(id: PatId) -> Self {
        ids::PatId(id.0 as u64)
    }
}
impl From<OAuthClientId> for ids::OAuthClientId {
    fn from(id: OAuthClientId) -> Self {
        ids::OAuthClientId(id.0 as u64)
    }
}
impl From<ids::OAuthClientId> for OAuthClientId {
    fn from(id: ids::OAuthClientId) -> Self {
        Self(id.0 as i64)
    }
}
impl From<OAuthRedirectUriId> for ids::OAuthRedirectUriId {
    fn from(id: OAuthRedirectUriId) -> Self {
        ids::OAuthRedirectUriId(id.0 as u64)
    }
}
impl From<OAuthClientAuthorizationId> for ids::OAuthClientAuthorizationId {
    fn from(id: OAuthClientAuthorizationId) -> Self {
        ids::OAuthClientAuthorizationId(id.0 as u64)
    }
}

impl From<ids::PayoutId> for PayoutId {
    fn from(id: ids::PayoutId) -> Self {
        PayoutId(id.0 as i64)
    }
}
impl From<PayoutId> for ids::PayoutId {
    fn from(id: PayoutId) -> Self {
        ids::PayoutId(id.0 as u64)
    }
}

impl From<ids::ProductId> for ProductId {
    fn from(id: ids::ProductId) -> Self {
        ProductId(id.0 as i64)
    }
}
impl From<ProductId> for ids::ProductId {
    fn from(id: ProductId) -> Self {
        ids::ProductId(id.0 as u64)
    }
}
impl From<ids::ProductPriceId> for ProductPriceId {
    fn from(id: ids::ProductPriceId) -> Self {
        ProductPriceId(id.0 as i64)
    }
}
impl From<ProductPriceId> for ids::ProductPriceId {
    fn from(id: ProductPriceId) -> Self {
        ids::ProductPriceId(id.0 as u64)
    }
}

impl From<ids::UserSubscriptionId> for UserSubscriptionId {
    fn from(id: ids::UserSubscriptionId) -> Self {
        UserSubscriptionId(id.0 as i64)
    }
}
impl From<UserSubscriptionId> for ids::UserSubscriptionId {
    fn from(id: UserSubscriptionId) -> Self {
        ids::UserSubscriptionId(id.0 as u64)
    }
}

impl From<ids::ChargeId> for ChargeId {
    fn from(id: ids::ChargeId) -> Self {
        ChargeId(id.0 as i64)
    }
}
impl From<ChargeId> for ids::ChargeId {
    fn from(id: ChargeId) -> Self {
        ids::ChargeId(id.0 as u64)
    }
}
