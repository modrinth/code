use crate::config::{MODRINTH_API_URL_V3, MODRINTH_SOCKET_URL};
use crate::data::ModrinthCredentials;
use crate::event::FriendPayload;
use crate::event::emit::emit_friend;
use crate::state::tunnel::InternalTunnelSocket;
use crate::state::{ProcessManager, Profile, TunnelSocket};
use crate::util::fetch::{FetchSemaphore, fetch_advanced, fetch_json};
use ariadne::ids::UserId;
use ariadne::networking::message::{
    ClientToServerMessage, ServerToClientMessage,
};
use ariadne::users::UserStatus;
use async_tungstenite::WebSocketStream;
use async_tungstenite::tokio::{ConnectStream, connect_async};
use async_tungstenite::tungstenite::Message;
use async_tungstenite::tungstenite::client::IntoClientRequest;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use either::Either;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use reqwest::Method;
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

pub(super) type WriteSocket =
    Arc<RwLock<Option<SplitSink<WebSocketStream<ConnectStream>, Message>>>>;
pub(super) type TunnelSockets = Arc<DashMap<Uuid, Arc<InternalTunnelSocket>>>;

pub struct FriendsSocket {
    write: WriteSocket,
    user_statuses: Arc<DashMap<UserId, UserStatus>>,
    tunnel_sockets: TunnelSockets,
}

#[derive(Deserialize, Serialize)]
pub struct UserFriend {
    pub id: String,
    pub friend_id: String,
    pub accepted: bool,
    pub created: DateTime<Utc>,
}

impl Default for FriendsSocket {
    fn default() -> Self {
        Self::new()
    }
}

impl FriendsSocket {
    pub fn new() -> Self {
        Self {
            write: Arc::new(RwLock::new(None)),
            user_statuses: Arc::new(DashMap::new()),
            tunnel_sockets: Arc::new(DashMap::new()),
        }
    }

    #[tracing::instrument(skip_all)]
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
                "{MODRINTH_SOCKET_URL}_internal/launcher_socket?code={}",
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
                        let mut write_lock = self.write.write().await;
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
                    let sockets = self.tunnel_sockets.clone();

                    tokio::spawn(async move {
                        let mut read_stream = read;
                        while let Some(msg_result) = read_stream.next().await {
                            match msg_result {
                                Ok(msg) => {
                                    let server_message = match msg {
                                        Message::Text(text) => {
                                            ServerToClientMessage::deserialize(
                                                Either::Left(&text),
                                            )
                                            .ok()
                                        }
                                        Message::Binary(bytes) => {
                                            ServerToClientMessage::deserialize(
                                                Either::Right(&bytes),
                                            )
                                            .ok()
                                        }
                                        Message::Ping(bytes) => {
                                            if let Some(write) = write_handle
                                                .write()
                                                .await
                                                .as_mut()
                                            {
                                                let _ = write
                                                    .send(Message::Pong(bytes))
                                                    .await;
                                            }

                                            continue;
                                        }
                                        Message::Pong(_)
                                        | Message::Frame(_) => continue,
                                        Message::Close(_) => break,
                                    };

                                    if let Some(server_message) = server_message
                                    {
                                        match server_message {
                                            ServerToClientMessage::StatusUpdate { status } => {
                                                statuses.insert(status.user_id, status.clone());
                                                let _ = emit_friend(FriendPayload::StatusUpdate { user_status: status }).await;
                                            },
                                            ServerToClientMessage::UserOffline { id } => {
                                                statuses.remove(&id);
                                                let _ = emit_friend(FriendPayload::UserOffline { id }).await;
                                            }
                                            ServerToClientMessage::FriendStatuses { statuses: new_statuses } => {
                                                statuses.clear();
                                                new_statuses.into_iter().for_each(|status| {
                                                    statuses.insert(status.user_id, status);
                                                });
                                                let _ = emit_friend(FriendPayload::StatusSync).await;
                                            }
                                            ServerToClientMessage::FriendRequest { from } => {
                                                let _ = emit_friend(FriendPayload::FriendRequest { from }).await;
                                            }
                                            ServerToClientMessage::FriendRequestRejected { .. } => todo!(),

                                            ServerToClientMessage::FriendSocketListening { .. } => {}, // TODO
                                            ServerToClientMessage::FriendSocketStoppedListening { .. } => {}, // TODO

                                            ServerToClientMessage::SocketConnected { to_socket, new_socket } => {
                                                if let Some(connected_to) = sockets.get(&to_socket) {
                                                    if let InternalTunnelSocket::Listening(local_addr) = *connected_to.value().clone() {
                                                        if let Ok(new_stream) = TcpStream::connect(local_addr).await {
                                                            let (read, write) = new_stream.into_split();
                                                            sockets.insert(new_socket, Arc::new(InternalTunnelSocket::Connected(Mutex::new(write))));
                                                            Self::socket_read_loop(write_handle.clone(), read, new_socket);
                                                            continue;
                                                        }
                                                    }
                                                }
                                                let _ = Self::send_message(&write_handle, ClientToServerMessage::SocketClose { socket: new_socket }).await;
                                            },
                                            ServerToClientMessage::SocketClosed { socket } => {
                                                sockets.remove_if(&socket, |_, x| matches!(*x.clone(), InternalTunnelSocket::Connected(_)));
                                            },
                                            ServerToClientMessage::SocketData { socket, data } => {
                                                if let Some(mut socket) = sockets.get_mut(&socket) {
                                                    if let InternalTunnelSocket::Connected(ref stream) = *socket.value_mut().clone() {
                                                        let _ = stream.lock().await.write_all(&data).await;
                                                    }
                                                }
                                            },
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Error handling message from websocket server: {:?}",
                                        e
                                    );
                                }
                            }
                        }

                        let mut w = write_handle.write().await;
                        *w = None;
                    });
                }
                Err(e) => {
                    tracing::error!(
                        "Error connecting to friends socket: {e:?}"
                    );

                    return Err(crate::Error::from(e));
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn socket_loop() -> crate::Result<()> {
        let state = crate::State::get().await?;

        tokio::task::spawn(async move {
            let mut last_connection = Utc::now();
            let mut last_ping = Utc::now();

            loop {
                let connected = state.friends_socket.is_connected().await;

                if !connected
                    && Utc::now().signed_duration_since(last_connection)
                        > chrono::Duration::seconds(30)
                {
                    last_connection = Utc::now();
                    last_ping = Utc::now();
                    let _ = state
                        .friends_socket
                        .connect(
                            &state.pool,
                            &state.api_semaphore,
                            &state.process_manager,
                        )
                        .await;
                } else if connected
                    && Utc::now().signed_duration_since(last_ping)
                        > chrono::Duration::seconds(10)
                {
                    last_ping = Utc::now();
                    let mut write = state.friends_socket.write.write().await;
                    if let Some(write) = write.as_mut() {
                        let _ = write.send(Message::Ping(Bytes::new())).await;
                    }
                }

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn disconnect(&self) -> crate::Result<()> {
        let mut write_lock = self.write.write().await;
        if let Some(ref mut write_half) = *write_lock {
            write_half.close().await?;
            *write_lock = None;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_status(
        &self,
        profile_name: Option<String>,
    ) -> crate::Result<()> {
        Self::send_message(
            &self.write,
            ClientToServerMessage::StatusUpdate { profile_name },
        )
        .await
    }

    #[tracing::instrument(skip_all)]
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

    #[tracing::instrument(skip(self))]
    pub fn friend_statuses(&self) -> Vec<UserStatus> {
        self.user_statuses
            .iter()
            .map(|x| x.value().clone())
            .collect()
    }

    #[tracing::instrument(skip(exec, semaphore))]
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

    #[tracing::instrument(skip(exec, semaphore))]
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

    #[tracing::instrument(skip(self))]
    pub async fn open_port(&self, port: u16) -> crate::Result<TunnelSocket> {
        let socket_id = Uuid::new_v4();
        let socket = self.tunnel_sockets.entry(socket_id).insert(Arc::new(
            InternalTunnelSocket::Listening(SocketAddr::new(
                "127.0.0.1".parse().unwrap(),
                port,
            )),
        ));
        Self::send_message(
            &self.write,
            ClientToServerMessage::SocketListen { socket: socket_id },
        )
        .await?;
        self.create_tunnel_socket(socket_id, socket)
    }

    pub async fn is_connected(&self) -> bool {
        self.write.read().await.is_some()
    }

    fn create_tunnel_socket(
        &self,
        socket_id: Uuid,
        socket: impl Deref<Target = Arc<InternalTunnelSocket>>,
    ) -> crate::Result<TunnelSocket> {
        Ok(TunnelSocket {
            socket_id,
            write: self.write.clone(),
            sockets: self.tunnel_sockets.clone(),
            internal: socket.clone(),
        })
    }

    fn socket_read_loop(
        write: WriteSocket,
        mut read_half: OwnedReadHalf,
        socket_id: Uuid,
    ) {
        tokio::spawn(async move {
            let mut read_buffer = [0u8; 8192];
            loop {
                match read_half.read(&mut read_buffer).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let _ = Self::send_message(
                            &write,
                            ClientToServerMessage::SocketSend {
                                socket: socket_id,
                                data: read_buffer[..n].to_vec(),
                            },
                        )
                        .await;
                    }
                };
            }
        });
    }

    #[tracing::instrument(skip(write))]
    pub(super) async fn send_message(
        write: &WriteSocket,
        message: ClientToServerMessage,
    ) -> crate::Result<()> {
        let serialized = match message.serialize()? {
            Either::Left(text) => Message::text(text),
            Either::Right(bytes) => Message::binary(bytes),
        };

        let mut write_lock = write.write().await;
        if let Some(ref mut write_half) = *write_lock {
            write_half.send(serialized).await?;
        }

        Ok(())
    }
}
