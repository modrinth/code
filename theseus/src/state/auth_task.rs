use crate::{
    hydra::{self, init::AuthInit},
    launcher::auth::Credentials,
};
use tokio::sync::RwLock;

use tokio::task::JoinHandle;

// Authentication task
// A wrapper over the authentication task that allows it to be called from the frontend
// without caching the task handle in the frontend

pub struct AuthTask {
    pub http_server: RwLock<Option<tiny_http::Server>>,
    pub task_handle: RwLock<Option<JoinHandle<crate::Result<Credentials>>>>,
}

impl AuthTask {
    pub fn new() -> AuthTask {
        AuthTask {
            http_server: RwLock::new(None),
            task_handle: RwLock::new(None),
        }
    }

    pub async fn begin_auth() -> crate::Result<AuthInit> {
        let state = crate::State::get().await?;
        // Init task, get url
        let login = hydra::init::init().await;

        {
            let read = state.auth_flow.http_server.read().await;
            if read.is_none() {
                drop(read);
                let mut write = state.auth_flow.http_server.write().await;
                *write =
                    Some(tiny_http::Server::http("0.0.0.0:20123").map_err(
                        |err| {
                            crate::ErrorKind::HydraError(format!(
                                "Could not start local server: {}",
                                err
                            ))
                        },
                    )?)
            }
        }

        // Await completion
        let task = tokio::spawn(hydra::complete::wait_finish());

        // Flow is going, store in state and return
        let mut write = state.auth_flow.task_handle.write().await;
        *write = Some(task);

        Ok(login)
    }

    pub async fn await_auth_completion() -> crate::Result<Credentials> {
        // Gets the task handle from the state, replacing with None
        let task = {
            let state = crate::State::get().await?;
            let mut write = state.auth_flow.task_handle.write().await;

            write.take()
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
            let mut write = state.auth_flow.task_handle.write().await;

            write.take()
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
