use serde::{Deserialize, Serialize};

pub mod program_monitor;

pub type RuleID = u64;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub name: String,
    pub is_active: bool,
    pub module: Module,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Module {
    ProgramMonitor(program_monitor::Rule),
}
