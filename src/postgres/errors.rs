use std::fmt;
use std::error::Error;
use std::fmt::{Debug, Formatter};


pub(super) trait ErrorGenerator<E> {
    fn generate_error(&self, msg: String) -> E;
}

#[derive(Debug)]
pub(crate) enum JoinTableError {
    InputInconsistentError(String),
    InputInvalidError(String),
}

impl fmt::Display for JoinTableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputInconsistentError(e) => write!(f, "Error occurred during parsing the collection input in preparing join table process due to {}", e),
            Self::InputInvalidError(e) => write!(f, "Error occurred during validating the input data in preparing join table process due to {}", e),
        }
    }
}

impl Error for JoinTableError {}

pub(super) struct JoinTableErrorGenerator;

impl ErrorGenerator<JoinTableError> for JoinTableErrorGenerator {
    fn generate_error(&self, msg: String) -> JoinTableError {
        JoinTableError::InputInvalidError(msg)
    }
}

#[derive(Debug)]
pub(crate) enum ConditionError {
    InputInvalidError(String),
}

impl fmt::Display for ConditionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputInvalidError(e) => write!(f, "Error occurred during validating the input data in condition prepare process due to {}", e),
        }
    }
}

impl Error for ConditionError {}

pub(super) struct ConditionErrorGenerator;

impl ErrorGenerator<ConditionError> for ConditionErrorGenerator {
    fn generate_error(&self, msg: String) -> ConditionError {
        ConditionError::InputInvalidError(msg)
    }
}

#[derive(Debug)]
pub(crate) enum QueryColumnError {
    InputInvalidError(String),
    InputInconsistentError(String),
}

impl fmt::Display for QueryColumnError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputInvalidError(e) => write!(f, "Error occurred during validating the input data in query column process due to {}", e),
            Self::InputInconsistentError(e) => write!(f, "Error occurred during query text build process in query column process due to {}", e),
        }
    }
}

impl Error for QueryColumnError {}

pub(super) struct QueryColumnErrorGenerator;

impl ErrorGenerator<QueryColumnError> for QueryColumnErrorGenerator {
    fn generate_error(&self, msg: String) -> QueryColumnError {
        QueryColumnError::InputInvalidError(msg)
    }
}

#[derive(Debug)]
pub(crate) enum UpdateSetError {
    InputInvalidError(String),
}

impl fmt::Display for UpdateSetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputInvalidError(e) => write!(f, "Error occurred during validating the input data in update values process due to {}", e),
        }
    }
}

impl Error for UpdateSetError {}

pub(super) struct UpdateSetErrorGenerator;
impl ErrorGenerator<UpdateSetError> for UpdateSetErrorGenerator {
    fn generate_error(&self, msg: String) -> UpdateSetError {
        UpdateSetError::InputInvalidError(msg)
    }
}

#[derive(Debug)]
pub(crate) enum InsertValueError {
    InputInvalidError(String),
    InputInconsistentError(String),
}

impl fmt::Display for InsertValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputInvalidError(e) => write!(f, "Error occurred during validating the input data in insert values process due to {}", e),
            Self::InputInconsistentError(e) => write!(f, "Error occurred during check the input data in insert values process due to {}", e),
        }
    }
}

impl Error for InsertValueError {}

pub(super) struct InsertValueErrorGenerator;

impl ErrorGenerator<InsertValueError> for InsertValueErrorGenerator {
    fn generate_error(&self, msg: String) -> InsertValueError {
        InsertValueError::InputInvalidError(msg)
    }
}

#[derive(Debug)]
pub(crate) enum PostgresBaseError {
    InputInvalidError(String),
    ConfigNotDefinedError(String),
    UnsafeExecutionError(String),
    UnexpectedError(String),
    ConnectionNotFoundError(String),
    SQLExecutionError(String),
    TokioPostgresError(String),
}

impl fmt::Display for PostgresBaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputInvalidError(e) => write!(f, "Error occurred during validating the input data in postgres execution process due to {}", e),
            Self::ConfigNotDefinedError(e) => write!(f, "Config doesn't exist in your environment arguments. {}", e),
            Self::UnsafeExecutionError(e) => write!(f, "Unsafe SQL execution is detected from {}.", e),
            Self::UnexpectedError(e) => write!(f, "Critical error occurred due to {}", e),
            Self::ConnectionNotFoundError(e) => write!(f, "SQL execution need connection but it can't be found. {}", e),
            Self::SQLExecutionError(e) => write!(f, "SQL execution failed due to {}", e),
            Self::TokioPostgresError(e) => write!(f, "Get error from tokio-postgres crate: {}", e),
        }
    }
}

impl Error for PostgresBaseError {}
