use crate::util::json::parse_object_async_reader;
use crate::{ErrorKind, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio::task::AbortHandle;
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
        let socket = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?;

        let addr = socket.local_addr()?;
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        let waiting_responses = Arc::new(Mutex::new(HashMap::new()));

        let join_handle = {
            let waiting_responses = waiting_responses.clone();
            tokio::spawn(async move {
                if let Err(e) = RpcServer::run(
                    socket,
                    message_receiver,
                    self.handlers,
                    waiting_responses.clone(),
                )
                .await
                {
                    tracing::error!("Failed to run RPC server: {e}");
                }
                waiting_responses.lock().unwrap().clear();
            })
        };
        Ok(RpcServer {
            port: addr.port(),
            message_sender,
            waiting_responses,
            abort_handle: join_handle.abort_handle(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct RpcServer {
    port: u16,
    message_sender: mpsc::UnboundedSender<RpcMessage>,
    waiting_responses: WaitingResponsesMap,
    abort_handle: AbortHandle,
}

impl RpcServer {
    pub fn port(&self) -> u16 {
        self.port
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

    async fn run(
        socket: TcpListener,
        mut message_receiver: mpsc::UnboundedReceiver<RpcMessage>,
        handlers: HandlerMap,
        waiting_responses: WaitingResponsesMap,
    ) -> Result<()> {
        let (mut socket, _) = socket.accept().await?;
        loop {
            tokio::select! {
                message = message_receiver.recv() => {
                    let Some(message) = message else {
                        break;
                    };
                    tracing::debug!("Sending RPC message {message:?}");
                    let json = serde_json::to_vec(&message)?;
                    socket.write_all(&json).await?;
                    socket.flush().await?;
                },
                message = parse_object_async_reader(&mut socket) => {
                    let message: RpcMessage = message?;
                    tracing::debug!("Received RPC message {message:?}");
                    if let RpcMessageBody::Call { method, args } = message.body {
                        let response = match handlers.get(method.as_str()) {
                            Some(handler) => match handler(args).await {
                                Ok(result) => RpcMessageBody::Respond {
                                    response: result,
                                },
                                Err(e) => RpcMessageBody::Error {
                                    error: e.to_string(),
                                },
                            }
                            None => RpcMessageBody::Error {
                                error: format!("Unknown theseus RPC method {method}"),
                            },
                        };
                        let json = serde_json::to_vec(&RpcMessage {
                            id: message.id,
                            body: response,
                        })?;
                        socket.write_all(&json).await?;
                        socket.flush().await?;
                    } else if let Some(sender) = waiting_responses.lock().unwrap().remove(&message.id) {
                        let _ = sender.send(match message.body {
                            RpcMessageBody::Respond { response } => Ok(response),
                            RpcMessageBody::Error { error } => Err(ErrorKind::RpcError(error).into()),
                            _ => unreachable!(),
                        });
                    }
                },
            }
        }
        Ok(())
    }
}

impl Drop for RpcServer {
    fn drop(&mut self) {
        self.abort_handle.abort();
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
