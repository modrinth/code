use modrinth_app_analytics_sdk::types::enums::{AppArch, AppOs};
use modrinth_app_analytics_sdk::types::event::AppEventType;
use modrinth_app_analytics_sdk::{AppAnalytics, AppAnalyticsParameters};
use sqlx::SqlitePool;
use tracing::info;
use url::Url;
use uuid::Uuid;

/// Endpoint the app sends batches of analytics events to.
const PUSH_URL: &str =
    concat!(env!("MODRINTH_API_BASE_URL"), "analytics/app-events");

/// Starts the analytics client, opted out when `telemetry` is disabled.
///
/// The stored installation ID is created when `telemetry` is enabled and
/// deleted when it is disabled, on every platform.
///
/// Returns `None` when the app runs on an OS or CPU architecture analytics
/// events can't describe, or when the client can't be set up. Analytics never
/// block the app from starting.
pub(crate) async fn init(
    pool: &SqlitePool,
    telemetry: bool,
) -> Option<AppAnalytics> {
    let installation_id = match sync_installation_id(pool, telemetry).await {
        Ok(installation_id) => installation_id,
        Err(error) => {
            tracing::warn!("Failed to set up analytics: {error}");
            return None;
        }
    };

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
            installation_id,
            user_agent: crate::launcher_user_agent(),
            os,
            arch,
            app_version: env!("CARGO_PKG_VERSION").into(),
        })
    };

    match params.await {
        Ok(params) => {
            let a = AppAnalytics::new(params).await;
            info!(analytics.url = PUSH_URL, "Initialized app analytics");
            a.add_event(modrinth_app_analytics_sdk::create_event(
                "Generic",
                AppEventType::AppStart {
                    arch,
                    os,
                    app_version: env!("CARGO_PKG_VERSION").into(),
                },
            ));
            Some(a)
        }
        Err(error) => {
            tracing::warn!("Failed to set up analytics: {error}");
            None
        }
    }
}

/// Replaces this installation's analytics ID with a new UUIDv4 and returns
/// it.
pub(crate) async fn reset_installation_id(
    pool: &SqlitePool,
) -> crate::Result<Uuid> {
    let new_id = Uuid::new_v4();
    let new_id_str = new_id.to_string();

    sqlx::query!(
        "
        INSERT INTO app_analytics (id, installation_id)
        VALUES (0, $1)
        ON CONFLICT (id) DO UPDATE
        SET installation_id = excluded.installation_id
        ",
        new_id_str,
    )
    .execute(pool)
    .await?;

    Ok(new_id)
}

/// Creates the installation ID if `telemetry` is enabled, or deletes it if
/// not, and returns the ID that now exists.
pub(crate) async fn sync_installation_id(
    pool: &SqlitePool,
    telemetry: bool,
) -> crate::Result<Option<Uuid>> {
    if telemetry {
        get_or_create_installation_id(pool).await.map(Some)
    } else {
        delete_installation_id(pool).await?;
        Ok(None)
    }
}

async fn delete_installation_id(pool: &SqlitePool) -> crate::Result<()> {
    sqlx::query!(
        "
        DELETE FROM app_analytics
        WHERE id = 0
        ",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Returns this installation's analytics ID, creating it if there is none.
///
/// The ID is a UUIDv4. A stored ID that isn't one is replaced.
async fn get_or_create_installation_id(
    pool: &SqlitePool,
) -> crate::Result<Uuid> {
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
