use tauri::Runtime;
use theseus::auth::Credentials;
use theseus::{auth, State};
use tokio::sync::oneshot;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    TheseusError(#[from] theseus::Error),
    #[error("Couldn't create a new window: {0}")]
    TauriError(#[from] tauri::Error),
    #[error("Couldn't get the URL: {0}")]
    RecvError(#[from] oneshot::error::RecvError),
    #[error("Tokyo task failed to execute to completion: {0}")]
    TokioTaskJoinError(#[from] tokio::task::JoinError),
    #[error("Failed to parse the UUID: {0}")]
    UuidParseError(#[from] uuid::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn auth_user_add<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<(), Error> {
    let (tx, rx) = oneshot::channel::<url::Url>();
    let flow = tokio::spawn(auth::authenticate(tx));

    let url = rx.await?;
    let _window = tauri::WindowBuilder::new(
        &app,
        "login",
        tauri::WindowUrl::External(url),
    )
    .title("Login to Microsoft")
    .build()?;

    let _credentials = flow.await??;
    State::sync().await?;
    Ok(())
}

#[tauri::command]
pub async fn auth_users() -> Result<Box<[Credentials]>, Error> {
    let creds = auth::users().await?;
    Ok(creds)
}

#[tauri::command]
pub async fn auth_remove_user(user: String) -> Result<(), Error> {
    Ok(auth::remove_user(uuid::Uuid::parse_str(&user)?).await?)
}
