use std::{any::type_name, str::FromStr, sync::LazyLock};

use eyre::{Context, eyre};

macro_rules! vars {
    (
        $(
            $field:ident: $ty:ty
        ),* $(,)?
    ) => {
        #[derive(Debug)]
        pub struct EnvVars {
            $(
                #[expect(non_snake_case, reason = "environment variables are UPPER_SNAKE_CASE")]
                pub $field: $ty,
            )*
        }

        pub static ENV_VARS: LazyLock<EnvVars> = LazyLock::new(|| {
            let mut err = eyre!("failed to read environment variables");

            $(
                let $field: Option<$ty> = match parse_value::<$ty>(stringify!($field)) {
                    Ok(value) => Some(value),
                    Err(source) => {
                        err = err.wrap_err(source);
                        None
                    }
                };
            )*

            EnvVars {
                $(
                    $field: $field.unwrap_or_else(|| panic!("{err:?}")),
                )*
            }
        });
    };
}

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
}
