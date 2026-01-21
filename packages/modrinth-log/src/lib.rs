#![doc = include_str!("../README.md")]

use std::str::FromStr;

use eyre::{Result, WrapErr, eyre};
#[cfg(feature = "sentry")]
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_ecs::ECSLayerBuilder;
use tracing_subscriber::{
    EnvFilter, layer::SubscriberExt, util::SubscriberInitExt,
};

/// How this service will output logs to the terminal output.
///
/// See [`init`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum OutputFormat {
    /// Human-readable format using [`tracing_subscriber::fmt::layer`].
    #[default]
    Human,
    /// Elastic Common Schema JSON output using [`ECSLayerBuilder`].
    Json,
}

impl FromStr for OutputFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            _ => Err(()),
        }
    }
}

/// Key for the environment variable that determines the output format.
pub const OUTPUT_FORMAT_ENV_VAR: &str = "MODRINTH_OUTPUT_FORMAT";

/// Initializes logging for Modrinth services.
///
/// This uses [`OUTPUT_FORMAT_ENV_VAR`] to determine the [`OutputFormat`] to
/// use - see that type for details of each possible format.
///
/// # Errors
///
/// Errors if logging could not be initialized.
pub fn init() -> Result<()> {
    init_with_config(false)
}

/// Initializes logging for Modrinth services.
///
/// This uses [`OUTPUT_FORMAT_ENV_VAR`] to determine the [`OutputFormat`] to
/// use - see that type for details of each possible format.
///
/// - `compact`: if using [`OutputFormat::Human`], logs will not show timestamps
///   or target details.
///
/// # Errors
///
/// Errors if logging could not be initialized.
pub fn init_with_config(compact: bool) -> Result<()> {
    let output_format = match env_var(OUTPUT_FORMAT_ENV_VAR) {
        Ok(format) => format
            .parse::<OutputFormat>()
            .map_err(|_| eyre!("invalid output format '{format}'"))?,
        Err(_) => OutputFormat::Human,
    };

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let (layer1, layer2, layer3) = match (output_format, compact) {
        (OutputFormat::Human, false) => {
            (Some(tracing_subscriber::fmt::layer()), None, None)
        }
        (OutputFormat::Human, true) => (
            None,
            Some(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_target(false),
            ),
            None,
        ),
        (OutputFormat::Json, _) => {
            (None, None, Some(ECSLayerBuilder::default().stdout()))
        }
    };

    let registry = tracing_subscriber::registry();

    let registry = registry
        .with(env_filter)
        .with(layer1)
        .with(layer2)
        .with(layer3);

    // Add Sentry layer but don't capture any events from tracing events, just breadcrumbs
    #[cfg(feature = "sentry")]
    let registry = registry.with(
        sentry::integrations::tracing::SentryLayer::default().event_filter(
            |metadata| match *metadata.level() {
                Level::ERROR | Level::WARN | Level::INFO => {
                    sentry::integrations::tracing::EventFilter::Breadcrumb
                }
                Level::DEBUG | Level::TRACE => {
                    sentry::integrations::tracing::EventFilter::empty()
                }
            },
        ),
    );

    registry
        .try_init()
        .wrap_err("failed to initialize tracing registry")?;

    Ok(())
}

fn env_var(key: &str) -> Result<String> {
    let value = dotenvy::var(key)
        .wrap_err_with(|| eyre!("missing environment variable `{key}`"))?;
    if value.is_empty() {
        Err(eyre!("environment variable `{key}` is empty"))
    } else {
        Ok(value)
    }
}
