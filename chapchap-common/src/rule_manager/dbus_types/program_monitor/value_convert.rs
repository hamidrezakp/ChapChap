use serde::{Deserialize, Serialize};
use zbus::zvariant::{Error, OwnedValue, Type, Value};

#[derive(Debug, Serialize, Deserialize, Type, Clone, Value)]
struct ActionStructed {
    case: u8,
    inner: OwnedValue,
}

impl<'a> From<super::Action> for Value<'a> {
    fn from(value: super::Action) -> Self {
        let (case, inner) = match value {
            super::Action::BlockProgramExecution(inner) => (0, inner.into()),
        };

        ActionStructed { case, inner }.into()
    }
}

impl<'a: 'static> TryFrom<Value<'a>> for super::Action {
    type Error = Error;

    fn try_from(value: Value<'a>) -> Result<Self, Self::Error> {
        let tmp: ActionStructed = value.try_into().clone()?;

        match tmp.case {
            0 => {
                let inner: u64 = tmp.inner.try_into()?;
                Ok(super::Action::BlockProgramExecution(inner))
            }
            _ => Err(Error::IncorrectType),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, Value)]
struct FilterStructed {
    case: u8,
    inner: OwnedValue,
}

impl<'a> From<super::Filter> for Value<'a> {
    fn from(value: super::Filter) -> Self {
        let (case, inner) = match value {
            super::Filter::Basic(inner) => (0, inner),
            super::Filter::Scheduled(inner) => (1, inner),
            super::Filter::TimeLimited(inner) => (2, inner),
        };

        FilterStructed { case, inner }.into()
    }
}

impl<'a: 'static> TryFrom<Value<'a>> for super::Filter {
    type Error = Error;

    fn try_from(value: Value<'a>) -> Result<Self, Self::Error> {
        let tmp: FilterStructed = value.try_into()?;

        match tmp.case {
            0 => Ok(super::Filter::Basic(tmp.inner)),
            1 => Ok(super::Filter::TimeLimited(tmp.inner)),
            2 => Ok(super::Filter::Scheduled(tmp.inner)),
            _ => Err(Error::IncorrectType),
        }
    }
}
