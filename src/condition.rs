use std::collections::HashSet;
use std::fmt::{Display, Formatter, Result as FmtResult};

use super::domain::Property;
use super::error::Result;
use super::value::Value;

#[derive(Debug, Clone)]
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

    pub fn variable(&self) -> Pid {
        self.variable
    }

    pub fn eval(&self, actual: &Value) -> Result<bool> {
        self.variable.validate(&actual)?;
        Ok(&self.expected == actual)
    }
}

#[derive(Debug, Clone)]
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
            write!(f, "{}, ", item)?;
        }
        write!(f, "]")
    }
}

impl<Pid: Property> In<Pid> {
    pub fn new(variable: Pid, expected: HashSet<Value>) -> Result<Self> {
        for item in expected.iter() {
            variable.validate(&item)?;
        }
        Ok(In { variable, expected })
    }

    pub fn variable(&self) -> Pid {
        self.variable
    }

    pub fn eval(&self, actual: &Value) -> Result<bool> {
        self.variable.validate(&actual)?;
        Ok(self.expected.contains(&actual))
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{error, value};
    use crate::testproperty::Property;

    #[test]
    fn is_positive() {
        let is = Is::<Property>::new(Property::Int, Value::Int(42)).unwrap();
        assert!(is.eval(&Value::Int(42)).unwrap());
    }

    #[test]
    fn is_negative() {
        let is = Is::<Property>::new(Property::Int, Value::Int(42)).unwrap();
        assert!(!is.eval(&Value::Int(24)).unwrap());
    }

    #[test]
    fn is_mismatch_new() {
        let is = Is::<Property>::new(Property::Int, Value::Bool(false));
        assert!(is.is_err());
        assert!(matches!(
            is.unwrap_err(),
            error::Error::TypeMismatch("Property::Int", value::Datatype::Int, value::Datatype::Bool)
        ));
    }

    #[test]
    fn is_mismatch_eval() {
        let is = Is::<Property>::new(Property::Int, Value::Int(42)).unwrap();
        let result = is.eval(&Value::Str("hello".to_string()));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            error::Error::TypeMismatch("Property::Int", value::Datatype::Int, value::Datatype::Str)
        ));
    }

    #[test]
    fn in_positive() {
        let values = vec![Value::Int(41), Value::Int(42)];
        let isin = In::<Property>::new(Property::Int, values.into_iter().collect()).unwrap();
        assert!(isin.eval(&Value::Int(42)).unwrap());
    }

    #[test]
    fn in_negative() {
        let values = vec![Value::Int(41), Value::Int(21)];
        let isin = In::<Property>::new(Property::Int, values.into_iter().collect()).unwrap();
        assert!(!isin.eval(&Value::Int(24)).unwrap());
    }

    #[test]
    fn in_mismatch_new() {
        let values = vec![Value::Int(42), Value::Str("in".to_owned())];
        let isin = In::<Property>::new(Property::Int, values.into_iter().collect());

        assert!(isin.is_err());
        assert!(matches!(
            isin.unwrap_err(),
            error::Error::TypeMismatch("Property::Int", value::Datatype::Int, value::Datatype::Str)
        ));
    }

    #[test]
    fn in_mismatch_eval() {
        let values = vec![Value::Str("is".to_owned()), Value::Str("in".to_owned())];
        let isin = In::<Property>::new(Property::Str, values.into_iter().collect()).unwrap();

        let result = isin.eval(&Value::Bool(true));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            error::Error::TypeMismatch("Property::Str", value::Datatype::Str, value::Datatype::Bool)
        ));
    }
}
