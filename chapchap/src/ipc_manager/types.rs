#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Rule maanger: {0}")]
    RuleManager(crate::rule_manager::Error),
}
