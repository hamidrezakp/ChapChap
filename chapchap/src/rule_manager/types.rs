use chapchap_common::rule_manager::RuleID;
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum Error {
    #[error("Rule not found")]
    RuleNotFound,

    #[error("Rule already exists: {0}")]
    RuleAlreadyExist(RuleID),

    #[error("Internal: {0}")]
    Internal(String),
}
