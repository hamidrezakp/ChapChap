use serde::{Deserialize, Serialize};
use zbus::{zvariant::Type, DBusError};

mod program_monitor;

#[derive(Debug, Serialize, Deserialize, DBusError)]
pub enum Error {
    InvalidZVariant(String),
    Internal,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct Rule {
    pub name: String,
    pub is_active: bool,
    pub module: Module,
}

impl From<super::types::Rule> for Rule {
    fn from(value: super::types::Rule) -> Self {
        Self {
            name: value.name,
            is_active: value.is_active,
            module: value.module.into(),
        }
    }
}

impl TryFrom<Rule> for super::types::Rule {
    type Error = Error;

    fn try_from(value: Rule) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            is_active: value.is_active,
            module: value.module.try_into()?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum Module {
    ProgramMonitor(program_monitor::Rule),
}

impl From<super::types::Module> for Module {
    fn from(value: super::types::Module) -> Self {
        match value {
            super::types::Module::ProgramMonitor(i) => Self::ProgramMonitor(i.into()),
        }
    }
}

impl TryFrom<Module> for super::types::Module {
    type Error = Error;

    fn try_from(value: Module) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            Module::ProgramMonitor(i) => Self::ProgramMonitor(i.try_into()?),
        })
    }
}
