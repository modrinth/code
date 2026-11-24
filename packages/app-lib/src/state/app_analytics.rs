use modrinth_app_analytics_sdk::types::enums::{AppArch, AppOs};
use modrinth_app_analytics_sdk::{AppAnalytics, AppAnalyticsParameters};
use sqlx::SqlitePool;
use url::Url;
use uuid::Uuid;

/// Endpoint the app sends batches of analytics events to.
const PUSH_URL: &str =
    concat!(env!("MODRINTH_API_BASE_URL"), "analytics/app-events");

/// Starts the analytics client, opted out when `telemetry` is disabled.
///
/// Returns `None` when the app runs on an OS or CPU architecture analytics
/// events can't describe, or when the client can't be set up. Analytics never
/// block the app from starting.
pub(crate) async fn init(
    pool: &SqlitePool,
    telemetry: bool,
) -> Option<AppAnalytics> {
    let (Some(os), Some(arch)) = (AppOs::current(), AppArch::current()) else {
        tracing::info!(
            os = std::env::consts::OS,
            arch = std::env::consts::ARCH,
            "Analytics are not supported on this platform"
        );
        return None;
    };

    let params = async {
        crate::Result::Ok(AppAnalyticsParameters {
            push_url: Url::parse(PUSH_URL)?,
            opted_out: !telemetry,
            user_agent: crate::launcher_user_agent(),
            installation_id: installation_id(pool).await?,
            os,
            arch,
            app_version: env!("CARGO_PKG_VERSION").into(),
        })
    };

    match params.await {
        Ok(params) => Some(AppAnalytics::new(params).await),
        Err(error) => {
            tracing::warn!("Failed to set up analytics: {error}");
            None
        }
    }
}

/// Returns this installation's analytics ID, creating it on first use.
///
/// The ID is a UUIDv4. A stored ID that isn't one is replaced.
async fn installation_id(pool: &SqlitePool) -> crate::Result<Uuid> {
    let new_id = Uuid::new_v4();
    let new_id_str = new_id.to_string();

    sqlx::query!(
        "
        INSERT INTO app_analytics (id, installation_id)
        VALUES (0, $1)
        ON CONFLICT (id) DO NOTHING
        ",
        new_id_str,
    )
    .execute(pool)
    .await?;

    let row = sqlx::query!(
        "
        SELECT installation_id
        FROM app_analytics
        WHERE id = 0
        ",
    )
    .fetch_one(pool)
    .await?;

    match Uuid::parse_str(&row.installation_id) {
        Ok(id) if id.get_version_num() == 4 => Ok(id),
        _ => {
            sqlx::query!(
                "
                UPDATE app_analytics
                SET installation_id = $1
                WHERE id = 0
                ",
                new_id_str,
            )
            .execute(pool)
            .await?;

            Ok(new_id)
        }
    }
}
