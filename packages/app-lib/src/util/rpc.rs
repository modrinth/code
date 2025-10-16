use crate::prelude::tcp_listen_any_loopback;
use crate::{ErrorKind, Result};
use futures::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio::task::AbortHandle;
use tokio_util::codec::{Decoder, LinesCodec, LinesCodecError};
use uuid::Uuid;

type HandlerFuture = Pin<Box<dyn Send + Future<Output = Result<Value>>>>;
type HandlerMethod = Box<dyn Send + Sync + Fn(Vec<Value>) -> HandlerFuture>;
type HandlerMap = HashMap<&'static str, HandlerMethod>;
type WaitingResponsesMap =
    Arc<Mutex<HashMap<Uuid, oneshot::Sender<Result<Value>>>>>;

pub struct RpcServerBuilder {
    handlers: HandlerMap,
}

impl RpcServerBuilder {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    // We'll use this function in the future. Please remove this #[allow] when we do.
    #[allow(dead_code)]
    pub fn handler(
        mut self,
        function_name: &'static str,
        handler: HandlerMethod,
    ) -> Self {
        self.handlers.insert(function_name, Box::new(handler));
        self
    }

    pub async fn launch(self) -> Result<RpcServer> {
        let socket = tcp_listen_any_loopback().await?;
        let address = socket.local_addr()?;
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        let waiting_responses = Arc::new(Mutex::new(HashMap::new()));

        let join_handle = {
            let waiting_responses = waiting_responses.clone();
            tokio::spawn(async move {
                let mut server = RunningRpcServer {
                    message_receiver,
                    handlers: self.handlers,
                    waiting_responses: waiting_responses.clone(),
                };
                if let Err(e) = server.run(socket).await {
                    tracing::error!("Failed to run RPC server: {e}");
                }
                waiting_responses.lock().unwrap().clear();
            })
        };
        Ok(RpcServer {
            address,
            message_sender,
            waiting_responses,
            abort_handle: join_handle.abort_handle(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct RpcServer {
    address: SocketAddr,
    message_sender: mpsc::UnboundedSender<RpcMessage>,
    waiting_responses: WaitingResponsesMap,
    abort_handle: AbortHandle,
}

impl RpcServer {
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub async fn call_method<R: DeserializeOwned>(
        &self,
        method: &str,
    ) -> Result<R> {
        self.call_method_any(method, vec![]).await
    }

    pub async fn call_method_2<R: DeserializeOwned>(
        &self,
        method: &str,
        arg1: impl Serialize,
        arg2: impl Serialize,
    ) -> Result<R> {
        self.call_method_any(
            method,
            vec![serde_json::to_value(arg1)?, serde_json::to_value(arg2)?],
        )
        .await
    }

    async fn call_method_any<R: DeserializeOwned>(
        &self,
        method: &str,
        args: Vec<Value>,
    ) -> Result<R> {
        if self.message_sender.is_closed() {
            return Err(ErrorKind::RpcError(
                "RPC connection closed".to_string(),
            )
            .into());
        }

        let id = Uuid::new_v4();
        let (send, recv) = oneshot::channel();
        self.waiting_responses.lock().unwrap().insert(id, send);

        let message = RpcMessage {
            id,
            body: RpcMessageBody::Call {
                method: method.to_owned(),
                args,
            },
        };
        if self.message_sender.send(message).is_err() {
            self.waiting_responses.lock().unwrap().remove(&id);
            return Err(ErrorKind::RpcError(
                "RPC connection closed while sending".to_string(),
            )
            .into());
        }

        tracing::debug!("Waiting on result for {id}");
        let Ok(result) = recv.await else {
            self.waiting_responses.lock().unwrap().remove(&id);
            return Err(ErrorKind::RpcError(
                "RPC connection closed while waiting for response".to_string(),
            )
            .into());
        };
        result.and_then(|x| Ok(serde_json::from_value(x)?))
    }
}

impl Drop for RpcServer {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}

struct RunningRpcServer {
    message_receiver: mpsc::UnboundedReceiver<RpcMessage>,
    handlers: HandlerMap,
    waiting_responses: WaitingResponsesMap,
}

impl RunningRpcServer {
    async fn run(&mut self, listener: TcpListener) -> Result<()> {
        let (socket, _) = listener.accept().await?;
        drop(listener);

        let mut socket = LinesCodec::new().framed(socket);
        loop {
            let to_send = tokio::select! {
                message = self.message_receiver.recv() => {
                    if message.is_none() {
                        break;
                    }
                    message
                },
                message = socket.next() => {
                    let message: RpcMessage = match message {
                        None => break,
                        Some(Ok(message)) => serde_json::from_str(&message)?,
                        Some(Err(LinesCodecError::Io(e))) => Err(e)?,
                        Some(Err(LinesCodecError::MaxLineLengthExceeded)) => unreachable!(),
                    };
                    self.handle_message(message).await?
                },
            };
            if let Some(message) = to_send {
                let json = serde_json::to_string(&message)?;
                match socket.send(json).await {
                    Ok(()) => {}
                    Err(LinesCodecError::Io(e)) => Err(e)?,
                    Err(LinesCodecError::MaxLineLengthExceeded) => {
                        unreachable!()
                    }
                };
            }
        }
        Ok(())
    }

    async fn handle_message(
        &self,
        message: RpcMessage,
    ) -> Result<Option<RpcMessage>> {
        if let RpcMessageBody::Call { method, args } = message.body {
            let response = match self.handlers.get(method.as_str()) {
                Some(handler) => match handler(args).await {
                    Ok(result) => RpcMessageBody::Respond { response: result },
                    Err(e) => RpcMessageBody::Error {
                        error: e.to_string(),
                    },
                },
                None => RpcMessageBody::Error {
                    error: format!("Unknown theseus RPC method {method}"),
                },
            };
            Ok(Some(RpcMessage {
                id: message.id,
                body: response,
            }))
        } else if let Some(sender) =
            self.waiting_responses.lock().unwrap().remove(&message.id)
        {
            let _ = sender.send(match message.body {
                RpcMessageBody::Respond { response } => Ok(response),
                RpcMessageBody::Error { error } => {
                    Err(ErrorKind::RpcError(error).into())
                }
                _ => unreachable!(),
            });
            Ok(None)
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcMessage {
    id: Uuid,
    #[serde(flatten)]
    body: RpcMessageBody,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum RpcMessageBody {
    Call {
        method: String,
        args: Vec<Value>,
    },
    Respond {
        #[serde(default, skip_serializing_if = "Value::is_null")]
        response: Value,
    },
    Error {
        error: String,
    },
}
