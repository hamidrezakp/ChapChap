use serde::{Deserialize, Serialize};
use zbus::DBusError;

pub mod client;
pub mod dbus_types;

pub mod program_monitor;

pub type RuleID = u64;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub name: String,
    pub is_active: bool,
    pub module: Module,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RuleWithID {
    pub id: RuleID,
    pub rule: Rule,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Module {
    ProgramMonitor(program_monitor::Rule),
}

#[derive(Debug, DBusError)]
#[dbus_error(prefix = "ir.hrkp.Chapchap")]
pub enum Error {
    InvalidZVariant(String),
    RuleManager(String),

    #[dbus_error(zbus_error)]
    ZBus(zbus::Error),

    InternalServer(String),
}
