use crate::launcher::auth::Credentials;
use std::mem;
use tokio::task::JoinHandle;

// Authentication task
// A wrapper over the authentication task that allows it to be called from the frontend
// without caching the task handle in the frontend

pub struct AuthTask(Option<JoinHandle<crate::Result<Credentials>>>);

impl AuthTask {
    pub fn new() -> AuthTask {
        AuthTask(None)
    }

    pub async fn begin_auth() -> crate::Result<url::Url> {
        let state = crate::State::get().await?;

        // Creates a channel to receive the URL
        let (tx, rx) = tokio::sync::oneshot::channel::<url::Url>();
        let task = tokio::spawn(crate::auth::authenticate(tx));

        // If receiver is dropped, try to get Hydra error
        let url = rx.await;
        let url = match url {
            Ok(url) => url,
            Err(e) => {
                task.await??;
                return Err(e.into()); // truly a dropped receiver
            }
        };

        // Flow is going, store in state and return
        let mut write = state.auth_flow.write().await;
        write.0 = Some(task);

        Ok(url)
    }

    pub async fn await_auth_completion() -> crate::Result<Credentials> {
        // Gets the task handle from the state, replacing with None
        let task = {
            let state = crate::State::get().await?;
            let mut write = state.auth_flow.write().await;

            mem::replace(&mut write.0, None)
        };

        // Waits for the task to complete, and returns the credentials
        let credentials = task
            .ok_or(AuthTaskError::TaskMissing)?
            .await
            .map_err(AuthTaskError::from)??;

        Ok(credentials)
    }

    pub async fn cancel() -> crate::Result<()> {
        // Gets the task handle from the state, replacing with None
        let task = {
            let state = crate::State::get().await?;
            let mut write = state.auth_flow.write().await;

            mem::replace(&mut write.0, None)
        };
        if let Some(task) = task {
            // Cancels the task
            task.abort();
        }

        Ok(())
    }
}

impl Default for AuthTask {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AuthTaskError {
    #[error("Authentication task was aborted or missing")]
    TaskMissing,
    #[error("Join handle error")]
    JoinHandleError(#[from] tokio::task::JoinError),
}
