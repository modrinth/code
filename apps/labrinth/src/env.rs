use std::{any::type_name, str::FromStr, sync::LazyLock};

use eyre::{Context, eyre};
use rust_decimal::Decimal;

macro_rules! vars {
    (
        $(
            $field:ident: $ty:ty
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
                    let $field: Option<$ty> = match parse_value::<$ty>(stringify!($field)) {
                        Ok(value) => Some(value),
                        Err(source) => {
                            err = err.wrap_err(source);
                            None
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

fn parse_value<T>(key: &str) -> eyre::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    dotenvy::var(key)
        .wrap_err_with(|| eyre!("`{key}` missing"))?
        .parse::<T>()
        .wrap_err_with(|| {
            eyre!("`{key}` is not a valid `{}`", type_name::<T>())
        })
}

vars! {
    SENTRY_ENVIRONMENT: String,
    SENTRY_TRACES_SAMPLE_RATE: String,
    SITE_URL: String,
    CDN_URL: String,
    LABRINTH_ADMIN_KEY: String,
    LABRINTH_EXTERNAL_NOTIFICATION_KEY: String,
    RATE_LIMIT_IGNORE_KEY: String,
    DATABASE_URL: String,
    MEILISEARCH_READ_ADDR: String,
    MEILISEARCH_WRITE_ADDRS: String,
    MEILISEARCH_KEY: String,
    REDIS_URL: String,
    BIND_ADDR: String,
    SELF_ADDR: String,

    LOCAL_INDEX_INTERVAL: usize,
    VERSION_INDEX_INTERVAL: usize,

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

    ARCHON_URL: String,

    MURALPAY_API_URL: String,
    MURALPAY_API_KEY: String,
    MURALPAY_TRANSFER_API_KEY: String,
    MURALPAY_SOURCE_ACCOUNT_ID: String,

    DEFAULT_AFFILIATE_REVENUE_SPLIT: Decimal,
}
