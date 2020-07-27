use strum_macros::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Datatype {
    Bool,
    Int,
    Str,
}


#[derive(Debug, PartialEq, Eq, Hash, Display)]
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
