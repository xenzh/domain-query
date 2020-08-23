use std::result::Result as StdResult;

use strum::ParseError;
use thiserror::Error as ThisError;

use super::value::Datatype;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Property or Entity identifier was not found in the domain")]
    IdentifierNotFound(#[from] ParseError),

    #[error("Property type mismatch: property '{0}' is {1}, but provided value is {2}")]
    TypeMismatch(&'static str, Datatype, Datatype),

    #[error("Expression is empty")]
    ExpressionNoop,

    #[error("Operation reference {0}/{1} is out of bounds; expression: {2}")]
    ExpressionOutOfBounds(usize, usize, String),

    #[error("Operation reference {0} is invalid: it points to a operation that's not defined yet ({1}); expression: {2}")]
    ExpressionFutureReference(usize, usize, String),

    #[error("Expression is inconsistent: operation {0} ({1}) is not connected to the root ({2})")]
    ExpressionDisconnected(usize, String, String),
}

pub type Result<T> = StdResult<T, Error>;
