use std::str::FromStr;

use serde::{Deserialize, Serialize};
use zbus::zvariant::Error as VariantError;
use zbus::zvariant::{Array, OwnedValue, Type, Value};

use crate::rule_manager::program_monitor as program_monitor_types;

mod value_convert;

#[derive(Debug, Serialize, Deserialize, Type, OwnedValue)]
pub struct Rule {
    pub filter: Filter,
    pub action: Action,
}

impl From<program_monitor_types::Rule> for Rule {
    fn from(value: program_monitor_types::Rule) -> Self {
        Self {
            filter: value.filter.into(),
            action: value.action.into(),
        }
    }
}

impl TryFrom<Rule> for program_monitor_types::Rule {
    type Error = zbus::Error;

    fn try_from(value: Rule) -> Result<Self, Self::Error> {
        let filter = value.filter.try_into()?;
        let action = value.action.into();

        Ok(Self { filter, action })
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Value)]
pub struct TimeSlice {
    start: NaiveTime,
    end: NaiveTime,
}

impl From<program_monitor_types::TimeSlice> for TimeSlice {
    fn from(value: program_monitor_types::TimeSlice) -> Self {
        Self {
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}

impl TryFrom<TimeSlice> for program_monitor_types::TimeSlice {
    type Error = zbus::Error;

    fn try_from(value: TimeSlice) -> Result<Self, Self::Error> {
        let start = value.start.try_into()?;
        let end = value.end.try_into()?;
        Ok(Self { start, end })
    }
}

// We have to use `OwnedValue` because `zvariant` doesn't support enums with different number and
// type of cases
#[derive(Debug, Serialize, Deserialize, Type)]
pub enum Filter {
    Basic(OwnedValue),       // ()
    Scheduled(OwnedValue),   // Vec<TimeSlice>
    TimeLimited(OwnedValue), // Duration
}

impl From<program_monitor_types::Filter> for Filter {
    fn from(value: program_monitor_types::Filter) -> Self {
        match value {
            program_monitor_types::Filter::Basic => Self::Basic(OwnedValue::from(0u8)),
            program_monitor_types::Filter::Scheduled(items) => {
                let items: Array = items
                    .into_iter()
                    .map(Into::<TimeSlice>::into)
                    .collect::<Vec<TimeSlice>>()
                    .into();
                Self::Scheduled(items.into())
            }
            program_monitor_types::Filter::TimeLimited(d) => {
                let secs = d.as_secs();
                let nanos = d.subsec_nanos() as u64;
                let value: Array = vec![secs, nanos].into();
                Filter::TimeLimited(value.into())
            }
        }
    }
}

impl TryFrom<Filter> for program_monitor_types::Filter {
    type Error = zbus::Error;

    fn try_from(value: Filter) -> Result<Self, Self::Error> {
        match value {
            Filter::Basic(_) => Ok(program_monitor_types::Filter::Basic),
            Filter::Scheduled(i) => {
                let items = Array::try_from(i.to_owned())
                    .and_then(TryInto::<Vec<TimeSlice>>::try_into)
                    .map_err(|_| VariantError::IncorrectType)?
                    .into_iter()
                    .map(TryInto::<program_monitor_types::TimeSlice>::try_into)
                    .collect::<Result<_, _>>()?;
                Ok(program_monitor_types::Filter::Scheduled(items))
            }
            Filter::TimeLimited(i) => {
                let array: Array = i
                    .try_into()
                    .map_err(|_| zbus::Error::Variant(VariantError::IncorrectType))?;
                let value: Vec<u64> = array
                    .try_into()
                    .map_err(|_| zbus::Error::Variant(VariantError::IncorrectType))?;

                if value.len() != 2 {
                    return Err(zbus::Error::Variant(VariantError::IncorrectType));
                }

                let secs = value[0];

                if value[1] > (u32::MAX as u64) {
                    return Err(zbus::Error::Variant(VariantError::IncorrectType));
                }
                let nanos = value[1] as u32;

                let dur = std::time::Duration::new(secs, nanos);
                Ok(program_monitor_types::Filter::TimeLimited(dur))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum Action {
    BlockProgramExecution(u64),
}

impl From<program_monitor_types::Action> for Action {
    fn from(value: program_monitor_types::Action) -> Self {
        match value {
            program_monitor_types::Action::BlockProgramExecution(i) => {
                Self::BlockProgramExecution(i)
            }
        }
    }
}

impl From<Action> for program_monitor_types::Action {
    fn from(value: Action) -> Self {
        match value {
            Action::BlockProgramExecution(i) => {
                program_monitor_types::Action::BlockProgramExecution(i)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Value)]
pub struct NaiveTime(String);

impl From<chrono::NaiveTime> for NaiveTime {
    fn from(value: chrono::NaiveTime) -> Self {
        Self(value.to_string())
    }
}

impl TryFrom<NaiveTime> for chrono::NaiveTime {
    type Error = zbus::Error;

    fn try_from(value: NaiveTime) -> Result<Self, Self::Error> {
        chrono::NaiveTime::from_str(&value.0)
            .map_err(|_| zbus::Error::Variant(VariantError::IncorrectType))
    }
}
