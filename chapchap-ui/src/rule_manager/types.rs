use chapchap_common::rule_manager::Error as RuleError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Dbus error: {0}")]
    DBus(String),

    #[error("Rule service: {0}")]
    RuleService(RuleError),
}

impl From<RuleError> for Error {
    fn from(value: RuleError) -> Self {
        Self::RuleService(value)
    }
}
