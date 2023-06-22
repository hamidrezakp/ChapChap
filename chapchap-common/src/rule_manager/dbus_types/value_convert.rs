use serde::{Deserialize, Serialize};
use zbus::zvariant::{Error, OwnedValue, Type, Value};

#[derive(Debug, Serialize, Deserialize, Type, Value)]
struct ModuleStructed {
    case: u8,
    inner: OwnedValue,
}

impl<'a> From<super::Module> for Value<'a> {
    fn from(value: super::Module) -> Self {
        let (case, inner) = match value {
            super::Module::ProgramMonitor(inner) => (0, inner.try_into().unwrap()),
        };

        ModuleStructed { case, inner }.into()
    }
}

impl<'a: 'static> TryFrom<Value<'a>> for super::Module {
    type Error = Error;

    fn try_from(value: Value<'a>) -> Result<Self, Self::Error> {
        let tmp: ModuleStructed = value.try_into()?;

        match tmp.case {
            0 => {
                let inner: super::program_monitor::Rule = tmp.inner.try_into()?;
                Ok(super::Module::ProgramMonitor(inner))
            }
            _ => Err(Error::IncorrectType),
        }
    }
}
