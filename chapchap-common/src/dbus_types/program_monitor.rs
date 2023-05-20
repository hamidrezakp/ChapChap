use std::str::FromStr;

use serde::{Deserialize, Serialize};
use zbus::zvariant::{Array, OwnedValue, Type, Value};

use super::Error;

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct Rule {
    pub filter: Filter,
    pub action: Action,
}

impl From<crate::types::program_monitor::Rule> for Rule {
    fn from(value: crate::types::program_monitor::Rule) -> Self {
        Self {
            filter: value.filter.into(),
            action: value.action.into(),
        }
    }
}

impl TryFrom<Rule> for crate::types::program_monitor::Rule {
    type Error = Error;

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

impl From<crate::types::program_monitor::TimeSlice> for TimeSlice {
    fn from(value: crate::types::program_monitor::TimeSlice) -> Self {
        Self {
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}

impl TryFrom<TimeSlice> for crate::types::program_monitor::TimeSlice {
    type Error = Error;

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

impl From<crate::types::program_monitor::Filter> for Filter {
    fn from(value: crate::types::program_monitor::Filter) -> Self {
        match value {
            crate::types::program_monitor::Filter::Basic => Self::Basic(OwnedValue::from(0u8)),
            crate::types::program_monitor::Filter::Scheduled(items) => {
                let items: Array = items
                    .into_iter()
                    .map(Into::<TimeSlice>::into)
                    .collect::<Vec<TimeSlice>>()
                    .into();
                Self::Scheduled(items.into())
            }
            crate::types::program_monitor::Filter::TimeLimited(d) => {
                let secs = d.as_secs();
                let nanos = d.subsec_nanos() as u64;
                let value: Array = vec![secs, nanos].into();
                Filter::TimeLimited(value.into())
            }
        }
    }
}

impl TryFrom<Filter> for crate::types::program_monitor::Filter {
    type Error = Error;

    fn try_from(value: Filter) -> Result<Self, Self::Error> {
        match value {
            Filter::Basic(_) => Ok(crate::types::program_monitor::Filter::Basic),
            Filter::Scheduled(i) => {
                let items = Array::try_from(i)
                    .and_then(TryInto::<Vec<TimeSlice>>::try_into)
                    .map_err(|e| Error::InvalidZVariant(format!("Expected TimeSlice: {e}")))?
                    .into_iter()
                    .map(TryInto::<crate::types::program_monitor::TimeSlice>::try_into)
                    .collect::<Result<_, _>>()?;
                Ok(crate::types::program_monitor::Filter::Scheduled(items))
            }
            Filter::TimeLimited(i) => {
                let array: Array = i
                    .try_into()
                    .map_err(|e| Error::InvalidZVariant(format!("invalid Array: {e}")))?;
                let value: Vec<u64> = array
                    .try_into()
                    .map_err(|e| Error::InvalidZVariant(format!("invalid Array: {e}")))?;

                if value.len() != 2 {
                    return Err(Error::InvalidZVariant(
                        "invalid duration, expected 2 fields".into(),
                    ));
                }

                let secs = value[0];

                if value[1] > (u32::MAX as u64) {
                    return Err(Error::InvalidZVariant(
                        "invalid duration, expected subsecond nanos".into(),
                    ));
                }
                let nanos = value[1] as u32;

                let dur = std::time::Duration::new(secs, nanos);
                Ok(crate::types::program_monitor::Filter::TimeLimited(dur))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum Action {
    BlockProgramExecution(u64),
}

impl From<crate::types::program_monitor::Action> for Action {
    fn from(value: crate::types::program_monitor::Action) -> Self {
        match value {
            crate::types::program_monitor::Action::BlockProgramExecution(i) => {
                Self::BlockProgramExecution(i)
            }
        }
    }
}

impl From<Action> for crate::types::program_monitor::Action {
    fn from(value: Action) -> Self {
        match value {
            Action::BlockProgramExecution(i) => {
                crate::types::program_monitor::Action::BlockProgramExecution(i)
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
    type Error = Error;

    fn try_from(value: NaiveTime) -> Result<Self, Self::Error> {
        chrono::NaiveTime::from_str(&value.0)
            .map_err(|e| Error::InvalidZVariant(format!("NaiveTime: {e}")))
    }
}
