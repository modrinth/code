use super::DatabaseError;
use crate::models::ids::{
    ChargeId, CollectionId, FileId, ImageId, NotificationId,
    OAuthAccessTokenId, OAuthClientAuthorizationId, OAuthClientId,
    OAuthRedirectUriId, OrganizationId, PatId, PayoutId, ProductId,
    ProductPriceId, ProjectId, ReportId, SessionId, TeamId, TeamMemberId,
    ThreadId, ThreadMessageId, UserSubscriptionId, VersionId,
};
use ariadne::ids::base62_impl::to_base62;
use ariadne::ids::{UserId, random_base62_rng, random_base62_rng_range};
use censor::Censor;
use paste::paste;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sqlx::sqlx_macros::Type;

const ID_RETRY_COUNT: usize = 20;

macro_rules! generate_ids {
    ($function_name:ident, $return_type:ident, $select_stmnt:expr) => {
        pub async fn $function_name(
            con: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ) -> Result<$return_type, DatabaseError> {
            let mut rng = ChaCha20Rng::from_entropy();
            let length = 8;
            let mut id = random_base62_rng(&mut rng, length);
            let mut retry_count = 0;
            let censor = Censor::Standard + Censor::Sex;

            // Check if ID is unique
            loop {
                let results = sqlx::query!($select_stmnt, id as i64)
                    .fetch_one(&mut **con)
                    .await?;

                if results.exists.unwrap_or(true)
                    || censor.check(&*to_base62(id))
                {
                    id = random_base62_rng(&mut rng, length);
                } else {
                    break;
                }

                retry_count += 1;
                if retry_count > ID_RETRY_COUNT {
                    return Err(DatabaseError::RandomId);
                }
            }

            Ok($return_type(id as i64))
        }
    };
}

macro_rules! generate_bulk_ids {
    ($function_name:ident, $return_type:ident, $select_stmnt:expr) => {
        pub async fn $function_name(
            count: usize,
            con: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        ) -> Result<Vec<$return_type>, DatabaseError> {
            let mut rng = rand::thread_rng();
            let mut retry_count = 0;

            // Check if ID is unique
            loop {
                let base = random_base62_rng_range(&mut rng, 1, 10) as i64;
                let ids =
                    (0..count).map(|x| base + x as i64).collect::<Vec<_>>();

                let results = sqlx::query!($select_stmnt, &ids)
                    .fetch_one(&mut **con)
                    .await?;

                if !results.exists.unwrap_or(true) {
                    return Ok(ids
                        .into_iter()
                        .map(|x| $return_type(x))
                        .collect());
                }

                retry_count += 1;
                if retry_count > ID_RETRY_COUNT {
                    return Err(DatabaseError::RandomId);
                }
            }
        }
    };
}

macro_rules! db_id_interface {
    ($id_struct:ident $(, generator: $generator_function:ident @ $db_table:expr, $(bulk_generator: $bulk_generator_function:ident,)?)?) => {
        paste! {
            #[derive(Copy, Clone, Debug, Type, Serialize, Deserialize, PartialEq, Eq, Hash)]
            #[sqlx(transparent)]
            pub struct [< DB $id_struct >](pub i64);

            impl From<$id_struct> for [< DB $id_struct >] {
                fn from(id: $id_struct) -> Self {
                    Self(id.0 as i64)
                }
            }
            impl From<[< DB $id_struct >]> for $id_struct {
                fn from(id: [< DB $id_struct >]) -> Self {
                    Self(id.0 as u64)
                }
            }

            $(
                generate_ids!(
                    $generator_function,
                    [< DB $id_struct >],
                    "SELECT EXISTS(SELECT 1 FROM " + $db_table + " WHERE id=$1)"
                );

                $(
                    generate_bulk_ids!(
                        $bulk_generator_function,
                        [< DB $id_struct >],
                        "SELECT EXISTS(SELECT 1 FROM " + $db_table + " WHERE id = ANY($1))"
                    );
                )?
            )?
        }
    };
}

macro_rules! short_id_type {
    ($name:ident) => {
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
        )]
        #[sqlx(transparent)]
        pub struct $name(pub i32);
    };
}

db_id_interface!(
    ChargeId,
    generator: generate_charge_id @ "charges",
);
db_id_interface!(
    CollectionId,
    generator: generate_collection_id @ "collections",
);
db_id_interface!(
    FileId,
    generator: generate_file_id @ "files",
);
db_id_interface!(
    ImageId,
    generator: generate_image_id @ "uploaded_images",
);
db_id_interface!(
    NotificationId,
    generator: generate_notification_id @ "notifications",
    bulk_generator: generate_many_notification_ids,
);
db_id_interface!(
    OAuthAccessTokenId,
    generator: generate_oauth_access_token_id @ "oauth_access_tokens",
);
db_id_interface!(
    OAuthClientAuthorizationId,
    generator: generate_oauth_client_authorization_id @ "oauth_client_authorizations",
);
db_id_interface!(
    OAuthClientId,
    generator: generate_oauth_client_id @ "oauth_clients",
);
db_id_interface!(
    OAuthRedirectUriId,
    generator: generate_oauth_redirect_id @ "oauth_client_redirect_uris",
);
db_id_interface!(
    OrganizationId,
    generator: generate_organization_id @ "organizations",
);
db_id_interface!(
    PatId,
    generator: generate_pat_id @ "pats",
);
db_id_interface!(
    PayoutId,
    generator: generate_payout_id @ "payouts",
);
db_id_interface!(
    ProductId,
    generator: generate_product_id @ "products",
);
db_id_interface!(
    ProductPriceId,
    generator: generate_product_price_id @ "products_prices",
);
db_id_interface!(
    ProjectId,
    generator: generate_project_id @ "mods",
);
db_id_interface!(
    ReportId,
    generator: generate_report_id @ "reports",
);
db_id_interface!(
    SessionId,
    generator: generate_session_id @ "sessions",
);
db_id_interface!(
    TeamId,
    generator: generate_team_id @ "teams",
);
db_id_interface!(
    TeamMemberId,
    generator: generate_team_member_id @ "team_members",
);
db_id_interface!(
    ThreadId,
    generator: generate_thread_id @ "threads",
);
db_id_interface!(
    ThreadMessageId,
    generator: generate_thread_message_id @ "threads_messages",
);
db_id_interface!(
    UserId,
    generator: generate_user_id @ "users",
);
db_id_interface!(
    UserSubscriptionId,
    generator: generate_user_subscription_id @ "users_subscriptions",
);
db_id_interface!(
    VersionId,
    generator: generate_version_id @ "versions",
);

short_id_type!(CategoryId);
short_id_type!(GameId);
short_id_type!(LinkPlatformId);
short_id_type!(LoaderFieldEnumId);
short_id_type!(LoaderFieldEnumValueId);
short_id_type!(LoaderFieldId);
short_id_type!(LoaderId);
short_id_type!(NotificationActionId);
short_id_type!(ProjectTypeId);
short_id_type!(ReportTypeId);
short_id_type!(StatusId);
