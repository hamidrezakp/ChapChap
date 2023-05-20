use serde::{Deserialize, Serialize};

use super::modules::program_monitor;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    #[error("Btf error: {0}")]
    Btf(String),

    #[error("Process monitor module: {0}")]
    ProcessMonitor(program_monitor::Error),

    #[error("Internal Error: {0}")]
    Internal(String),
}
