//! Authentication flow interface
use crate::launcher::auth as inner;
use tokio::sync::oneshot;

pub use inner::Credentials;

/// Authenticate a user with Hydra
/// To run this, you need to first spawn this function as a task, then
/// open a browser to the given URL and finally wait on the spawned future
/// with the ability to cancel in case the browser is closed before finishing
pub async fn authenticate(
    browser_url: oneshot::Sender<url::Url>,
) -> crate::Result<Credentials> {
    let mut flow = inner::HydraAuthFlow::new().await?;
    let url = flow.prepare_login_url().await?;
    browser_url.send(url).map_err(|url| {
        crate::Error::OtherError(format!(
            "Error sending browser url to parent: {url}"
        ))
    })?;
    flow.extract_credentials().await
}

/// Refresh some credentials using Hydra, if needed
pub async fn refresh(
    credentials: &mut Credentials,
    update_name: bool,
) -> crate::Result<()> {
    if chrono::offset::Utc::now() > credentials.expires {
        inner::refresh_credentials(credentials).await?;
        if update_name {
            inner::refresh_username(credentials).await?;
        }
    }
    Ok(())
}
