use super::condition::Condition;
use super::domain::Property;
use super::value::Value;
use super::error::Result;

pub type OpRef = usize;

pub enum Operation<Pid: Property> {
    Is(Is<Pid>),
    In,
    Not(OpRef),
    Or(OpRef, OpRef),
    And(OpRef, OpRef),
}

pub struct Expression<Pid: Property> {
    ops: Vec<Operation<Pid>>,
}

// builder with validity checks

impl<Pid: Property> Expression<Pid> {
    pub fn new() -> Expression<Pid> {
        Expression { ops: Vec::new() }
    }

    fn root(&self) -> Option<OpRef> {
        if self.ops.is_empty() {
            None
        } else {
            Some(self.ops.len() - 1)
        }
    }

    pub fn is(&mut self, variable: Pid, value: Value) -> Result<OpRef> {
        let cond = Is::new(variable, value)?;
        self.ops.push(Operation::Is(cond));
        Ok(self.ops.len() - 1)
    }

    pub fn is_in(&self, variable: Pid, values: Vec<Value>) -> Result<OpRef> {
        unimplemented!();
    }
}

impl<Pid: Property> Default for Expression<Pid> {
    fn default() -> Self {
        Expression::new()
    }
}
