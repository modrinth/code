use super::message::{ClientToServerMessage, ServerToClientMessage};
use either::Either;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerializationError {
    #[error("Failed to (de)serialize message: {0}")]
    SerializationFailed(#[from] serde_json::Error),

    #[error("Failed to (de)serialize binary message: {0}")]
    BinarySerializationFailed(#[from] serde_cbor::Error),
}

macro_rules! message_serialization {
    ($message_enum:ty $(,$binary_pattern:pat_param)* $(,)?) => {
        impl $message_enum {
            pub fn is_binary(&self) -> bool {
                match self {
                    $(
                        $binary_pattern => true,
                    )*
                    _ => false,
                }
            }

            pub fn serialize(
                &self,
            ) -> Result<Either<String, Vec<u8>>, SerializationError> {
                Ok(match self {
                    $(
                        $binary_pattern => Either::Right(serde_cbor::to_vec(self)?),
                    )*
                    _ => Either::Left(serde_json::to_string(self)?),
                })
            }

            pub fn deserialize(
                msg: Either<&str, &[u8]>,
            ) -> Result<Self, SerializationError> {
                Ok(match msg {
                    Either::Left(text) => serde_json::from_str(&text)?,
                    Either::Right(bytes) => serde_cbor::from_slice(&bytes)?,
                })
            }
        }
    };
}

message_serialization!(
    ClientToServerMessage,
    ClientToServerMessage::SocketSend { .. },
);
message_serialization!(
    ServerToClientMessage,
    ServerToClientMessage::SocketData { .. },
);
