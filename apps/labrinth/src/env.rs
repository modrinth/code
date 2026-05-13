use std::{any::type_name, convert::Infallible, str::FromStr, sync::LazyLock};

use derive_more::{Deref, DerefMut};
use eyre::{Context, eyre};
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;

macro_rules! vars {
    (
        $(
            $field:ident: $ty:ty $(= $default:expr)?;
        )*
    ) => {
        #[derive(Debug)]
        #[allow(
            non_snake_case,
            reason = "environment variables are UPPER_SNAKE_CASE",
        )]
        pub struct EnvVars {
            $(
                pub $field: $ty,
            )*
        }

        impl EnvVars {
            pub fn from_env() -> eyre::Result<Self> {
                let mut err = eyre!("failed to read environment variables");

                $(
                    #[expect(
                        non_snake_case,
                        reason = "environment variables are UPPER_SNAKE_CASE",
                    )]
                    #[allow(
                        unused_assignments,
                        unused_mut,
                        reason = "`default` is not used if there is no default",
                    )]
                    let $field: Option<$ty> = {
                        let mut default = None::<$ty>;
                        $( default = Some({ $default }.into()); )?

                        match parse_value::<$ty>(stringify!($field), default) {
                            Ok(value) => Some(value),
                            Err(source) => {
                                err = err.wrap_err(eyre!("{source:#}"));
                                None
                            }
                        }
                    };
                )*

                Ok(EnvVars {
                    $(
                        $field: match $field {
                            Some(value) => value,
                            None => return Err(err),
                        },
                    )*
                })
            }
        }
    };
}

pub static ENV: LazyLock<EnvVars> = LazyLock::new(|| {
    EnvVars::from_env().unwrap_or_else(|err| panic!("{err:?}"))
});

fn parse_value<T>(key: &str, default: Option<T>) -> eyre::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match (dotenvy::var(key), default) {
        (Ok(value), _) => value.parse::<T>().wrap_err_with(|| {
            eyre!("`{key}` is not a valid `{}`", type_name::<T>())
        }),
        (Err(_), Some(default)) => Ok(default),
        (Err(_), None) => Err(eyre!("`{key}` missing")),
    }
}

pub fn init() -> eyre::Result<()> {
    dotenvy::dotenv().ok();
    EnvVars::from_env()?;
    LazyLock::force(&ENV);
    Ok(())
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Deref, DerefMut,
)]
pub struct Json<T: DeserializeOwned>(pub T);

impl<T: DeserializeOwned> FromStr for Json<T> {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map(Self)
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Deref, DerefMut,
)]
pub struct StringCsv(pub Vec<String>);

impl FromStr for StringCsv {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        Ok(Self(v))
    }
}

vars! {
    SENTRY_ENVIRONMENT: String = "development";
    SENTRY_TRACES_SAMPLE_RATE: f32 = 0.1f32;
    SITE_URL: String = "http://localhost:3000";
    CDN_URL: String = "file:///tmp/modrinth";
    LABRINTH_ADMIN_KEY: String = "";
    LABRINTH_MEDAL_KEY: String = "";
    LABRINTH_EXTERNAL_NOTIFICATION_KEY: String = "";
    RATE_LIMIT_IGNORE_KEY: String = "";
    DATABASE_URL: String = "postgresql://labrinth:labrinth@localhost/labrinth";
    REDIS_URL: String = "redis://localhost";
    BIND_ADDR: String = "";
    SELF_ADDR: String = "";

    LOCAL_INDEX_INTERVAL: u64 = 3600u64;
    VERSION_INDEX_INTERVAL: u64 = 1800u64;

    WHITELISTED_MODPACK_DOMAINS: Json<Vec<String>> = Json(vec![
        "cdn.modrinth.com".into(),
        "github.com".into(),
        "raw.githubusercontent.com".into(),
    ]);
    ALLOWED_CALLBACK_URLS: Json<Vec<String>> = Json(vec![
        "localhost".into(),
        ".modrinth.com".into(),
        "127.0.0.1".into(),
        "[::1]".into(),
    ]);
    ANALYTICS_ALLOWED_ORIGINS: Json<Vec<String>> = Json(vec![
        "http://127.0.0.1:3000".into(),
        "http://localhost:3000".into(),
        "https://modrinth.com".into(),
        "https://www.modrinth.com".into(),
        "*".into(),
    ]);

    // search
    SEARCH_BACKEND: crate::search::SearchBackendKind = crate::search::SearchBackendKind::Typesense;
    SEARCH_INDEX_CHUNK_SIZE: i64 = 5000i64;
    TYPESENSE_URL: String = "http://localhost:8108";
    TYPESENSE_API_KEY: String = "modrinth";
    TYPESENSE_INDEX_PREFIX: String = "labrinth";

    // storage
    STORAGE_BACKEND: crate::file_hosting::FileHostKind = crate::file_hosting::FileHostKind::Local;

    // s3
    S3_PUBLIC_BUCKET_NAME: String = "";
    S3_PUBLIC_USES_PATH_STYLE_BUCKET: bool = false;
    S3_PUBLIC_REGION: String = "";
    S3_PUBLIC_URL: String = "";
    S3_PUBLIC_ACCESS_TOKEN: String = "";
    S3_PUBLIC_SECRET: String = "";

    S3_PRIVATE_BUCKET_NAME: String = "";
    S3_PRIVATE_USES_PATH_STYLE_BUCKET: bool = false;
    S3_PRIVATE_REGION: String = "";
    S3_PRIVATE_URL: String = "";
    S3_PRIVATE_ACCESS_TOKEN: String = "";
    S3_PRIVATE_SECRET: String = "";

    // local
    MOCK_FILE_PATH: String = "/tmp/modrinth";

    GITHUB_CLIENT_ID: String = "none";
    GITHUB_CLIENT_SECRET: String = "none";
    GITLAB_CLIENT_ID: String = "none";
    GITLAB_CLIENT_SECRET: String = "none";
    DISCORD_CLIENT_ID: String = "none";
    DISCORD_CLIENT_SECRET: String = "none";
    MICROSOFT_CLIENT_ID: String = "none";
    MICROSOFT_CLIENT_SECRET: String = "none";
    GOOGLE_CLIENT_ID: String = "none";
    GOOGLE_CLIENT_SECRET: String = "none";
    STEAM_API_KEY: String = "none";

    TREMENDOUS_API_URL: String = "https://testflight.tremendous.com/api/v2/";
    TREMENDOUS_API_KEY: String = "none";
    TREMENDOUS_PRIVATE_KEY: String = "none";

    PAYPAL_API_URL: String = "https://api-m.sandbox.paypal.com/v1/";
    PAYPAL_WEBHOOK_ID: String = "none";
    PAYPAL_CLIENT_ID: String = "none";
    PAYPAL_CLIENT_SECRET: String = "none";
    PAYPAL_NVP_USERNAME: String = "none";
    PAYPAL_NVP_PASSWORD: String = "none";
    PAYPAL_NVP_SIGNATURE: String = "none";

    PAYPAL_BALANCE_ALERT_THRESHOLD: u64 = 0u64;
    BREX_BALANCE_ALERT_THRESHOLD: u64 = 0u64;
    TREMENDOUS_BALANCE_ALERT_THRESHOLD: u64 = 0u64;
    MURAL_BALANCE_ALERT_THRESHOLD: u64 = 0u64;

    HCAPTCHA_SECRET: String = "none";

    SMTP_USERNAME: String = "";
    SMTP_PASSWORD: String = "";
    SMTP_HOST: String = "localhost";
    SMTP_PORT: u16 = 1025u16;
    SMTP_TLS: String = "none";
    SMTP_FROM_NAME: String = "Modrinth";
    SMTP_FROM_ADDRESS: String = "no-reply@mail.modrinth.com";

    SITE_VERIFY_EMAIL_PATH: String = "auth/verify-email";
    SITE_RESET_PASSWORD_PATH: String = "auth/reset-password";
    SITE_BILLING_PATH: String = "none";

    SENDY_URL: String = "none";
    SENDY_LIST_ID: String = "none";
    SENDY_API_KEY: String = "none";

    CLICKHOUSE_REPLICATED: bool = false;
    CLICKHOUSE_URL: String = "http://localhost:8123";
    CLICKHOUSE_USER: String = "default";
    CLICKHOUSE_PASSWORD: String = "default";
    CLICKHOUSE_DATABASE: String = "staging_ariadne";

    FLAME_ANVIL_URL: String = "none";

    GOTENBERG_URL: String = "http://localhost:13000";
    GOTENBERG_CALLBACK_BASE: String = "http://host.docker.internal:8000/_internal/gotenberg";
    GOTENBERG_TIMEOUT: u64 = 30000u64;

    STRIPE_API_KEY: String = "none";
    STRIPE_WEBHOOK_SECRET: String = "none";

    ADITUDE_API_KEY: String = "none";

    PYRO_API_KEY: String = "none";

    BREX_API_URL: String = "https://platform.brexapis.com/v2/";
    BREX_API_KEY: String = "none";

    DELPHI_URL: String = "";

    AVALARA_1099_API_URL: String = "https://www.track1099.com/api";
    AVALARA_1099_API_KEY: String = "none";
    AVALARA_1099_API_TEAM_ID: String = "none";
    AVALARA_1099_COMPANY_ID: String = "207337084";

    ANROK_API_URL: String = "";
    ANROK_API_KEY: String = "";

    PAYOUT_ALERT_SLACK_WEBHOOK: String = "none";
    CLOUDFLARE_INTEGRATION: bool = false;

    ARCHON_URL: String = "";

    MURALPAY_API_URL: String = "https://api-staging.muralpay.com";
    MURALPAY_API_KEY: String = "none";
    MURALPAY_TRANSFER_API_KEY: String = "none";
    MURALPAY_SOURCE_ACCOUNT_ID: muralpay::AccountId = muralpay::AccountId(uuid::Uuid::nil());

    DEFAULT_AFFILIATE_REVENUE_SPLIT: Decimal = Decimal::new(1, 1);

    DATABASE_ACQUIRE_TIMEOUT_MS: u64 = 30000u64;
    DATABASE_MIN_CONNECTIONS: u32 = 0u32;
    DATABASE_MAX_CONNECTIONS: u32 = 16u32;
    READONLY_DATABASE_URL: String = "";
    READONLY_DATABASE_MIN_CONNECTIONS: u32 = 0u32;
    READONLY_DATABASE_MAX_CONNECTIONS: u32 = 1u32;

    REDIS_WAIT_TIMEOUT_MS: u64 = 15000u64;
    REDIS_MAX_CONNECTIONS: u32 = 10000u32;
    REDIS_MIN_CONNECTIONS: usize = 0usize;

    SEARCH_OPERATION_TIMEOUT: u64 = 300000u64;

    SMTP_REPLY_TO_NAME: String = "";
    SMTP_REPLY_TO_ADDRESS: String = "";

    PUBLIC_DISCORD_WEBHOOK: String = "";
    MODERATION_SLACK_WEBHOOK: String = "";
    DELPHI_SLACK_WEBHOOK: String = "";

    TREMENDOUS_CAMPAIGN_ID: String = "none";

    // server pinging
    SERVER_PING_MAX_CONCURRENT: usize = 16usize;
    SERVER_PING_RETRIES: usize = 3usize;
    SERVER_PING_MIN_INTERVAL_SEC: u64 = 30u64 * 60;
    SERVER_PING_TIMEOUT_MS: u64 = 3u64 * 1000;
    SERVER_PING_MAX_FAIL_COUNT: u64 = 3u64;
}
