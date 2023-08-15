use crate::{
    hydra::{self, init::DeviceLoginSuccess},
    launcher::auth::Credentials,
};

use tokio::task::JoinHandle;

// Authentication task
// A wrapper over the authentication task that allows it to be called from the frontend
// without caching the task handle in the frontend

pub struct AuthTask(
    #[allow(clippy::type_complexity)]
    Option<JoinHandle<crate::Result<Credentials>>>,
);

impl AuthTask {
    pub fn new() -> AuthTask {
        AuthTask(None)
    }

    pub async fn begin_auth() -> crate::Result<DeviceLoginSuccess> {
        let state = crate::State::get().await?;
        // Init task, get url
        let login = hydra::init::init().await?;

        // Await completion
        let task = tokio::spawn(hydra::complete::wait_finish(
            login.device_code.clone(),
        ));

        // Flow is going, store in state and return
        let mut write = state.auth_flow.write().await;
        write.0 = Some(task);

        Ok(login)
    }

    pub async fn await_auth_completion() -> crate::Result<Credentials> {
        // Gets the task handle from the state, replacing with None
        let task = {
            let state = crate::State::get().await?;
            let mut write = state.auth_flow.write().await;

            write.0.take()
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

            write.0.take()
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
