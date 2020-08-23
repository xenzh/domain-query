use std::fmt::{Display, Formatter, Result as FmtResult};
use strum_macros::Display as StrumDisplay;

#[derive(Debug, Clone, Copy, PartialEq, Eq, StrumDisplay)]
pub enum Datatype {
    Bool,
    Int,
    Str,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Str(String),
}

impl Value {
    pub fn datatype(&self) -> Datatype {
        match *self {
            Value::Bool(_) => Datatype::Bool,
            Value::Int(_) => Datatype::Int,
            Value::Str(_) => Datatype::Str,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Value::Bool(val) => write!(f, "{}", val),
            Value::Int(val) => write!(f, "{}", val),
            Value::Str(ref val) => write!(f, "{}", val),
        }
    }
}
