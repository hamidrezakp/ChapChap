#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Dbus error: {0}")]
    DBus(zbus::Error),
}
