use crate::{domain, value};
use std::fmt::{Display, Formatter, Result as FmtResult};
use strum_macros::{EnumIter, EnumString};

#[derive(PartialEq, Clone, Copy, Hash, Eq, Debug, EnumIter, EnumString)]
pub enum Property {
    Bool,
    Int,
    Str,
}

impl Display for Property {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{}",
            match &self {
                Property::Bool => "Property::Bool",
                Property::Int => "Property::Int",
                Property::Str => "Property::Str",
            }
        )
    }
}

impl domain::DomainEnum for Property {}

impl domain::Property for Property {
    fn name(&self) -> &'static str {
        match &self {
            Property::Bool => "Property::Bool",
            Property::Int => "Property::Int",
            Property::Str => "Property::Str",
        }
    }

    fn datatype(&self) -> value::Datatype {
        match &self {
            Property::Bool => value::Datatype::Bool,
            Property::Int => value::Datatype::Int,
            Property::Str => value::Datatype::Str,
        }
    }
}
