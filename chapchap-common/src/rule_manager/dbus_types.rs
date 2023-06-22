use serde::{Deserialize, Serialize};
use zbus::zvariant::{Type, Value};

pub use super::RuleID;

use super::Error;

mod program_monitor;

mod value_convert;

#[derive(Debug, Serialize, Deserialize, Type, Value)]
pub struct Rule {
    pub name: String,
    pub is_active: bool,
    pub module: Module,
}

impl From<super::Rule> for Rule {
    fn from(value: super::Rule) -> Self {
        Self {
            name: value.name,
            is_active: value.is_active,
            module: value.module.into(),
        }
    }
}

impl TryFrom<Rule> for super::Rule {
    type Error = Error;

    fn try_from(value: Rule) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            is_active: value.is_active,
            module: value.module.try_into()?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Value)]
pub struct RuleWithID {
    pub id: RuleID,
    pub rule: Rule,
}

impl<'a> From<super::RuleWithID> for RuleWithID {
    fn from(value: super::RuleWithID) -> Self {
        Self {
            id: value.id,
            rule: value.rule.into(),
        }
    }
}

impl<'a> TryFrom<RuleWithID> for super::RuleWithID {
    type Error = Error;

    fn try_from(value: RuleWithID) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            rule: value.rule.try_into()?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum Module {
    ProgramMonitor(program_monitor::Rule),
}

impl From<super::Module> for Module {
    fn from(value: super::Module) -> Self {
        match value {
            super::Module::ProgramMonitor(i) => Self::ProgramMonitor(i.into()),
        }
    }
}

impl TryFrom<Module> for super::Module {
    type Error = Error;

    fn try_from(value: Module) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            Module::ProgramMonitor(i) => Self::ProgramMonitor(i.try_into()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::{program_monitor, Module, Rule, RuleWithID};
    use super::{Rule as DBUSRule, RuleWithID as DBUSRuleWithID};

    #[test]
    fn rule_into_and_from_dbus_type() {
        let rule = Rule {
            name: "Rule1".into(),
            is_active: true,
            module: Module::ProgramMonitor(program_monitor::Rule {
                filter: program_monitor::Filter::Basic,
                action: program_monitor::Action::BlockProgramExecution(100),
            }),
        };

        let dbus_rule: DBUSRule = rule.clone().into();

        let value: Rule = dbus_rule.try_into().unwrap();
        assert_eq!(value, rule);
    }

    #[test]
    fn rule_with_id_into_and_from_dbus_type() {
        let rule = RuleWithID {
            id: 0,
            rule: Rule {
                name: "Rule1".into(),
                is_active: true,
                module: Module::ProgramMonitor(program_monitor::Rule {
                    filter: program_monitor::Filter::Basic,
                    action: program_monitor::Action::BlockProgramExecution(100),
                }),
            },
        };

        let dbus_rule: DBUSRuleWithID = rule.clone().into();

        let value: RuleWithID = dbus_rule.try_into().unwrap();
        assert_eq!(value, rule);
    }
}
