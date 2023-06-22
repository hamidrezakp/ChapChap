use serde::{Deserialize, Serialize};

use super::modules::program_monitor;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    #[error("Btf error: {0}")]
    Btf(String),

    #[error("Program monitor module: {0}")]
    ProgramMonitor(program_monitor::Error),

    #[error("Module is not loaded: {0}")]
    ModuleNotLoaded(&'static str),

    #[error("Module already loaded: {0}")]
    ModuleAlreadyLoaded(&'static str),

    #[error("Internal Error: {0}")]
    Internal(String),
}
