use std::error::Error;
use std::fmt::{Display, Error, Formatter};

pub trait ErrorGenerator<E: Error> {
    fn generate_error(&self, msg: String) -> E;
}

#[derive(Debug, PartialEq)]
pub enum ConnectionConfigError {
    TypeError(String),
    UndefinedValueError(String),
}

impl Display for ConnectionConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeError(e) => write!(f, "TypeError occurred due to {}", e),
            Self::UndefinedValueError(e) => write!(f, "Undefined value referred due to {}", e),
        }
    }
}

impl Error for ConnectionConfigError {}
