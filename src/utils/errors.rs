use std::error::Error;
use std::fmt::{Display, Formatter};

pub trait ErrorGenerator<E: Error> {
    fn generate_error(&self, msg: String) -> E;
}

#[derive(Debug, PartialEq)]
pub enum ConnectionConfigError {
    TypeError(String),
    UndefinedValueError(String),
    ConnectionFailedError(String),
}

impl Display for ConnectionConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeError(e) => write!(f, "TypeError occurred due to {}", e),
            Self::UndefinedValueError(e) => write!(f, "Undefined value referred due to {}", e),
            Self::ConnectionFailedError(e) => write!(f, "Connection to PostgreSQL failed due to {}", e)
        }
    }
}

impl Error for ConnectionConfigError {}

#[derive(Debug, PartialEq)]
pub enum GeneratorError {
    InvalidTableNameError(String),
    InconsistentConfigError(String),
    InvalidInputError(String),
}

impl Display for GeneratorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTableNameError(e) => write!(f, "Table name is invalid due to {}", e),
            Self::InconsistentConfigError(e) => write!(f, "Configuration input is inconsistent due to {}", e),
            Self::InvalidInputError(e) => write!(f, "Input data is invalid due to {}", e),
        }
    }
}

impl Error for GeneratorError {}
