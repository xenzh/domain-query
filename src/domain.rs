use std::marker::PhantomData;
use std::str::FromStr;
use std::fmt::Display;

use strum::{IntoEnumIterator, ParseError};

use super::error::{Result, Error};
use super::value::{Datatype, Value};

pub trait DomainEnum: IntoEnumIterator + Copy + FromStr + Display {}

pub trait Property: DomainEnum {
    fn name(&self) -> &'static str;
    fn datatype(&self) -> Datatype;

    fn validate(&self, value: &Value) -> Result<()> {
        if self.datatype() != value.datatype() {
            Err(Error::TypeMismatch(self.name(), self.datatype(), value.datatype()))
        } else {
            Ok(())
        }
    }
}

pub trait Entity<Prop: Property>: DomainEnum {
    fn name(&self) -> &str;
    fn properties(&self) -> &[Prop];
}

pub trait Domain<Pid: Property, Eid: Entity<Pid>> {
    type Property = Pid;
    type Entity = Eid;
}

pub struct Lookup<Pid: Property, Eid: Entity<Pid>> {
    eid: PhantomData<Eid>,
    pid: PhantomData<Pid>,
}

impl<Pid: Property + FromStr<Err = ParseError>, Eid: Entity<Pid> + FromStr<Err = ParseError>>
    Lookup<Pid, Eid>
{
    pub fn property(name: &str) -> Result<Pid> {
        Ok(Pid::from_str(name)?)
    }

    pub fn entity(name: &str) -> Result<Eid> {
        Ok(Eid::from_str(name)?)
    }
}
