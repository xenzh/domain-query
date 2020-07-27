use std::result::Result as StdResult;

use thiserror::Error as ThisError;
use strum::ParseError;

use super::value::Datatype;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Property or Entity identifier was not found in the domain")]
    IdentifierNotFound(#[from] ParseError),

    #[error("Property type mismatch: property '{0}' is {1}, but provided value is {2}")]
    TypeMismatch(&'static str, Datatype, Datatype)
}

pub type Result<T> = StdResult<T, Error>;
