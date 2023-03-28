use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use thiserror::Error;

pub mod profile;
pub mod profile_create;

type Result<T> = std::result::Result<T, TheseusGuiError>;

// Main returnable Theseus GUI error
// Needs to be Serializable to be returned to the JavaScript side
#[derive(Error, Debug, Serialize)]
pub enum TheseusGuiError {
    #[error(transparent)]
    Serializable(TheseusSerializableError),

    #[error("Error creating profile {0}")]
    ProfileCreation(String),
}

// Serializable error intermediary, so TheseusGuiError can be Serializable (eg: so that we can return theseus::Errors in Tauri directly)
#[derive(Error, Debug)]
pub enum TheseusSerializableError {
    #[error("Theseus API error: {0}")]
    Theseus(#[from] theseus::Error),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}

// Generic implementation of From<T> for ErrorTypeA
impl<T> From<T> for TheseusGuiError
where
    TheseusSerializableError: From<T>,
{
    fn from(error: T) -> Self {
        TheseusGuiError::Serializable(TheseusSerializableError::from(error))
    }
}

// This is a very simple macro that implements a very basic Serializable for each variant of TheseusSerializableError,
// where the field is the string. (This allows easy extension to errors without many match arms)
macro_rules! impl_serialize {
    ($($variant:ident),* $(,)?) => {
        impl Serialize for TheseusSerializableError {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                match self {
                    $(
                        TheseusSerializableError::$variant(message) => {
                            let mut state = serializer.serialize_struct(stringify!($variant), 2)?;
                            state.serialize_field("field_name", stringify!($variant))?;
                            state.serialize_field("message", &message.to_string())?;
                            state.end()
                        },
                    )*
                }
            }
        }
    };
}

// Use the macro to implement Serialize for TheseusSerializableError
impl_serialize! {
    Theseus,
    IO,
}
