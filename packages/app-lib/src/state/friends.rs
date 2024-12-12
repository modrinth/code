use crate::config::{MODRINTH_API_URL_V3, MODRINTH_SOCKET_URL};
use crate::data::ModrinthCredentials;
use crate::event::emit::emit_friend;
use crate::event::FriendPayload;
use crate::state::{ProcessManager, Profile};
use crate::util::fetch::{fetch_advanced, fetch_json, FetchSemaphore};
use async_tungstenite::tokio::{connect_async, ConnectStream};
use async_tungstenite::tungstenite::client::IntoClientRequest;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::WebSocketStream;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use reqwest::header::HeaderValue;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

type WriteSocket =
    Arc<Mutex<Option<SplitSink<WebSocketStream<ConnectStream>, Message>>>>;

pub struct FriendsSocket {
    write: WriteSocket,
    user_statuses: Arc<DashMap<String, UserStatus>>,
}

#[derive(Deserialize, Serialize)]
pub struct UserFriend {
    pub id: String,
    // TODO: Remove this optional and serde alias on release
    pub friend_id: Option<String>,
    #[serde(alias = "pending")]
    pub accepted: bool,
    pub created: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientToServerMessage {
    StatusUpdate { profile_name: Option<String> },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerToClientMessage {
    StatusUpdate { status: UserStatus },
    UserOffline { id: String },
    FriendStatuses { statuses: Vec<UserStatus> },
    FriendRequest { from: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserStatus {
    pub user_id: String,
    pub profile_name: Option<String>,
    pub last_update: DateTime<Utc>,
}

impl Default for FriendsSocket {
    fn default() -> Self {
        Self::new()
    }
}

impl FriendsSocket {
    pub fn new() -> Self {
        Self {
            write: Arc::new(Mutex::new(None)),
            user_statuses: Arc::new(DashMap::new()),
        }
    }

    pub async fn connect(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
        semaphore: &FetchSemaphore,
        process_manager: &ProcessManager,
    ) -> crate::Result<()> {
        let credentials =
            ModrinthCredentials::get_and_refresh(exec, semaphore).await?;

        if let Some(credentials) = credentials {
            let mut request = format!(
                "{MODRINTH_SOCKET_URL}_internal/launcher_heartbeat?code={}",
                credentials.session
            )
            .into_client_request()?;

            let user_agent = format!(
                "modrinth/theseus/{} (support@modrinth.com)",
                env!("CARGO_PKG_VERSION")
            );
            request.headers_mut().insert(
                "User-Agent",
                HeaderValue::from_str(&user_agent).unwrap(),
            );

            let res = connect_async(request).await;

            match res {
                Ok((socket, _)) => {
                    tracing::info!("Connected to friends socket");
                    let (write, read) = socket.split();

                    {
                        let mut write_lock = self.write.lock().await;
                        *write_lock = Some(write);
                    }

                    if let Some(process) = process_manager.get_all().first() {
                        let profile =
                            Profile::get(&process.profile_path, exec).await?;

                        if let Some(profile) = profile {
                            let _ =
                                self.update_status(Some(profile.name)).await;
                        }
                    }

                    let write_handle = self.write.clone();
                    let statuses = self.user_statuses.clone();

                    tokio::spawn(async move {
                        let mut read_stream = read;
                        while let Some(msg_result) = read_stream.next().await {
                            match msg_result {
                                Ok(msg) => {
                                    let server_message = match msg {
                                        Message::Text(text) => {
                                            serde_json::from_str::<
                                                ServerToClientMessage,
                                            >(
                                                &text
                                            )
                                            .ok()
                                        }
                                        Message::Binary(bytes) => {
                                            serde_json::from_slice::<
                                                ServerToClientMessage,
                                            >(
                                                &bytes
                                            )
                                            .ok()
                                        }
                                        Message::Ping(_)
                                        | Message::Pong(_)
                                        | Message::Frame(_) => continue,
                                        Message::Close(_) => break,
                                    };

                                    if let Some(server_message) = server_message
                                    {
                                        match server_message {
                                            ServerToClientMessage::StatusUpdate { status } => {
                                                statuses.insert(status.user_id.clone(), status.clone());
                                                let _ = emit_friend(FriendPayload::StatusUpdate { user_status: status }).await;
                                            },
                                            ServerToClientMessage::UserOffline { id } => {
                                                statuses.remove(&id);
                                                let _ = emit_friend(FriendPayload::UserOffline { id }).await;
                                            }
                                            ServerToClientMessage::FriendStatuses { statuses: new_statuses } => {
                                                statuses.clear();
                                                new_statuses.into_iter().for_each(|status| {
                                                    statuses.insert(status.user_id.clone(), status);
                                                });
                                                let _ = emit_friend(FriendPayload::StatusSync).await;
                                            }
                                            ServerToClientMessage::FriendRequest { from } => {
                                                let _ = emit_friend(FriendPayload::FriendRequest { from }).await;
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("WebSocket error: {:?}", e);
                                    break;
                                }
                            }
                        }

                        let mut w = write_handle.lock().await;
                        *w = None;

                        Self::reconnect_task();
                    });
                }
                Err(e) => {
                    tracing::error!(
                        "Error connecting to friends socket: {e:?}"
                    );

                    Self::reconnect_task();

                    return Err(crate::Error::from(e));
                }
            }
        }

        Ok(())
    }

    fn reconnect_task() {
        tokio::task::spawn(async move {
            let res = async {
                let state = crate::State::get().await?;

                {
                    if state.friends_socket.write.lock().await.is_some() {
                        return Ok(());
                    }
                }

                state
                    .friends_socket
                    .connect(
                        &state.pool,
                        &state.api_semaphore,
                        &state.process_manager,
                    )
                    .await?;

                Ok::<(), crate::Error>(())
            };

            if let Err(e) = res.await {
                tracing::info!("Error reconnecting to friends socket: {e:?}");

                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                FriendsSocket::reconnect_task();
            }
        });
    }

    pub async fn disconnect(&self) -> crate::Result<()> {
        let mut write_lock = self.write.lock().await;
        if let Some(ref mut write_half) = *write_lock {
            write_half.close().await?;
            *write_lock = None;
        }
        Ok(())
    }

    pub async fn update_status(
        &self,
        profile_name: Option<String>,
    ) -> crate::Result<()> {
        let mut write_lock = self.write.lock().await;
        if let Some(ref mut write_half) = *write_lock {
            write_half
                .send(Message::Text(serde_json::to_string(
                    &ClientToServerMessage::StatusUpdate { profile_name },
                )?))
                .await?;
        }

        Ok(())
    }

    pub async fn friends(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
        semaphore: &FetchSemaphore,
    ) -> crate::Result<Vec<UserFriend>> {
        fetch_json(
            Method::GET,
            &format!("{MODRINTH_API_URL_V3}friends"),
            None,
            None,
            semaphore,
            exec,
        )
        .await
    }

    pub fn friend_statuses(&self) -> Vec<UserStatus> {
        self.user_statuses
            .iter()
            .map(|x| x.value().clone())
            .collect()
    }

    pub async fn add_friend(
        user_id: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
        semaphore: &FetchSemaphore,
    ) -> crate::Result<()> {
        fetch_advanced(
            Method::POST,
            &format!("{MODRINTH_API_URL_V3}friend/{user_id}"),
            None,
            None,
            None,
            None,
            semaphore,
            exec,
        )
        .await?;

        Ok(())
    }

    pub async fn remove_friend(
        user_id: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
        semaphore: &FetchSemaphore,
    ) -> crate::Result<()> {
        fetch_advanced(
            Method::DELETE,
            &format!("{MODRINTH_API_URL_V3}friend/{user_id}"),
            None,
            None,
            None,
            None,
            semaphore,
            exec,
        )
        .await?;

        Ok(())
    }
}
