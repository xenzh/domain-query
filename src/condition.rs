use std::collections::HashSet;
use std::fmt::{Display, Formatter, Result as FmtResult};

use super::domain::Property;
use super::error::Result;
use super::value::Value;

#[derive(Debug)]
pub struct Is<Pid: Property> {
    variable: Pid,
    expected: Value,
}

impl<Pid: Property> Display for Is<Pid> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{0} ({1}) == {2}",
            self.variable.name(),
            self.variable.datatype(),
            self.expected
        )
    }
}

impl<Pid: Property> Is<Pid> {
    pub fn new(variable: Pid, expected: Value) -> Result<Self> {
        variable.validate(&expected)?;
        Ok(Is { variable, expected })
    }

    pub fn eval(&self, actual: &Value) -> Result<bool> {
        self.variable.validate(&actual)?;
        Ok(&self.expected == actual)
    }
}

#[derive(Debug)]
pub struct In<Pid: Property> {
    variable: Pid,
    expected: HashSet<Value>,
}

impl<Pid: Property> Display for In<Pid> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{0} ({1}) in [",
            self.variable.name(),
            self.variable.datatype()
        )?;

        for item in &self.expected {
            write!(f, "{}", item)?;
        }
        write!(f, "]")
    }
}

impl<Pid: Property> In<Pid> {
    pub fn new(variable: Pid, expected: HashSet<Value>) -> Result<Self> {
        for item in &expected {
            variable.validate(item)?;
        }
        Ok(In { variable, expected })
    }

    pub fn eval(&self, actual: &Value) -> Result<bool> {
        self.variable.validate(&actual)?;
        Ok(self.expected.contains(&actual))
    }
}

pub enum Condition<Pid: Property> {
    Is(Is<Pid>),
    In(In<Pid>),
    Not,
    Or,
    And,
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{domain, error, value};
    use std::fmt::{Display, Formatter, Result as FmtResult};
    use strum_macros::{EnumIter, EnumString};

    #[derive(PartialEq, Clone, Copy, Debug, EnumIter, EnumString)]
    enum Property {
        One,
        Two,
    }

    impl Display for Property {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            write!(
                f,
                "{}",
                match &self {
                    Property::One => "one",
                    Property::Two => "two",
                }
            )
        }
    }

    impl domain::DomainEnum for Property {}

    impl domain::Property for Property {
        fn name(&self) -> &'static str {
            match &self {
                Property::One => "One",
                Property::Two => "Two",
            }
        }

        fn datatype(&self) -> value::Datatype {
            match &self {
                Property::One => value::Datatype::Int,
                Property::Two => value::Datatype::Str,
            }
        }
    }

    #[test]
    fn is_positive() {
        let is = Is::<Property>::new(Property::One, Value::Int(42)).unwrap();
        assert!(is.eval(&Value::Int(42)).unwrap());
    }

    #[test]
    fn is_negative() {
        let is = Is::<Property>::new(Property::One, Value::Int(42)).unwrap();
        assert!(!is.eval(&Value::Int(24)).unwrap());
    }

    #[test]
    fn is_mismatch_new() {
        let is = Is::<Property>::new(Property::One, Value::Bool(false));
        assert!(is.is_err());
        assert!(matches!(
            is.unwrap_err(),
            error::Error::TypeMismatch("One", value::Datatype::Int, value::Datatype::Bool)
        ));
    }

    #[test]
    fn is_mismatch_eval() {
        let is = Is::<Property>::new(Property::One, Value::Int(42)).unwrap();
        let result = is.eval(&Value::Str("hello".to_string()));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            error::Error::TypeMismatch("One", value::Datatype::Int, value::Datatype::Str)
        ));
    }
}
