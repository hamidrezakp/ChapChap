use chapchap_common::dbus_types;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<Error> for dbus_types::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::Internal(_) => Self::Internal,
        }
    }
}
