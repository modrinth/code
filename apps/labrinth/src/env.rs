use std::{any::type_name, convert::Infallible, str::FromStr, sync::LazyLock};

use derive_more::{Deref, DerefMut};
use eyre::{Context, eyre};
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;

macro_rules! vars {
    (
        $(
            $field:ident: $ty:ty $(= $default:expr)?
        ),* $(,)?
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
                                err = err.wrap_err(source);
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
    SENTRY_ENVIRONMENT: String,
    SENTRY_TRACES_SAMPLE_RATE: f32,
    SITE_URL: String,
    CDN_URL: String,
    LABRINTH_ADMIN_KEY: String,
    LABRINTH_MEDAL_KEY: String = "",
    LABRINTH_EXTERNAL_NOTIFICATION_KEY: String = "",
    RATE_LIMIT_IGNORE_KEY: String,
    DATABASE_URL: String,
    MEILISEARCH_READ_ADDR: String,
    MEILISEARCH_WRITE_ADDRS: StringCsv,
    MEILISEARCH_KEY: String,
    REDIS_URL: String,
    BIND_ADDR: String,
    SELF_ADDR: String,

    LOCAL_INDEX_INTERVAL: u64,
    VERSION_INDEX_INTERVAL: u64,

    WHITELISTED_MODPACK_DOMAINS: Json<Vec<String>>,
    ALLOWED_CALLBACK_URLS: Json<Vec<String>>,
    ANALYTICS_ALLOWED_ORIGINS: Json<Vec<String>>,

    STORAGE_BACKEND: crate::file_hosting::FileHostKind,

    GITHUB_CLIENT_ID: String,
    GITHUB_CLIENT_SECRET: String,
    GITLAB_CLIENT_ID: String,
    GITLAB_CLIENT_SECRET: String,
    DISCORD_CLIENT_ID: String,
    DISCORD_CLIENT_SECRET: String,
    MICROSOFT_CLIENT_ID: String,
    MICROSOFT_CLIENT_SECRET: String,
    GOOGLE_CLIENT_ID: String,
    GOOGLE_CLIENT_SECRET: String,
    STEAM_API_KEY: String,

    TREMENDOUS_API_URL: String,
    TREMENDOUS_API_KEY: String,
    TREMENDOUS_PRIVATE_KEY: String,

    PAYPAL_API_URL: String,
    PAYPAL_WEBHOOK_ID: String,
    PAYPAL_CLIENT_ID: String,
    PAYPAL_CLIENT_SECRET: String,
    PAYPAL_NVP_USERNAME: String,
    PAYPAL_NVP_PASSWORD: String,
    PAYPAL_NVP_SIGNATURE: String,

    HCAPTCHA_SECRET: String,

    SMTP_USERNAME: String,
    SMTP_PASSWORD: String,
    SMTP_HOST: String,
    SMTP_PORT: u16,
    SMTP_TLS: String,
    SMTP_FROM_NAME: String,
    SMTP_FROM_ADDRESS: String,

    SITE_VERIFY_EMAIL_PATH: String,
    SITE_RESET_PASSWORD_PATH: String,
    SITE_BILLING_PATH: String,

    SENDY_URL: String,
    SENDY_LIST_ID: String,
    SENDY_API_KEY: String,

    CLICKHOUSE_REPLICATED: bool,
    CLICKHOUSE_URL: String,
    CLICKHOUSE_USER: String,
    CLICKHOUSE_PASSWORD: String,
    CLICKHOUSE_DATABASE: String,

    FLAME_ANVIL_URL: String,

    GOTENBERG_URL: String,
    GOTENBERG_CALLBACK_BASE: String,
    GOTENBERG_TIMEOUT: u64,

    STRIPE_API_KEY: String,
    STRIPE_WEBHOOK_SECRET: String,

    ADITUDE_API_KEY: String,

    PYRO_API_KEY: String,

    BREX_API_URL: String,
    BREX_API_KEY: String,

    DELPHI_URL: String,

    AVALARA_1099_API_URL: String,
    AVALARA_1099_API_KEY: String,
    AVALARA_1099_API_TEAM_ID: String,
    AVALARA_1099_COMPANY_ID: String,

    ANROK_API_URL: String,
    ANROK_API_KEY: String,

    COMPLIANCE_PAYOUT_THRESHOLD: String,

    PAYOUT_ALERT_SLACK_WEBHOOK: String,
    CLOUDFLARE_INTEGRATION: bool = false,

    ARCHON_URL: String,

    MURALPAY_API_URL: String,
    MURALPAY_API_KEY: String,
    MURALPAY_TRANSFER_API_KEY: String,
    MURALPAY_SOURCE_ACCOUNT_ID: muralpay::AccountId,

    DEFAULT_AFFILIATE_REVENUE_SPLIT: Decimal,
}
