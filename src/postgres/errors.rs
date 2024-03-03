use std::fmt;
use std::error::Error;
use std::fmt::{Debug, Formatter};


pub(super) trait ErrorGenerator<E> {
    fn generate_error(&self, msg: String) -> E;
}

#[derive(Debug)]
pub(crate) enum StatementError {
    GenerationError(String),
    InputError(String),
}

impl fmt::Display for StatementError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::GenerationError(e) => write!(f, "Error occurred during statement generation process due to {}.", e),
            Self::InputError(e) => write!(f, "Error occurred from statement generator input is invalid on {}.", e),
        }
    }
}

impl Error for StatementError {}

#[derive(Debug)]
pub(crate) enum JoinTableError {
    InputInconsistentError(String),
    InputInvalidError(String),
}

impl fmt::Display for JoinTableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputInconsistentError(e) => write!(f, "Error occurred during parsing the collection input due to {}", e),
            Self::InputInvalidError(e) => write!(f, "Error occurred during validating the input data due to {}", e),
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
            Self::InputInvalidError(e) => write!(f, "Error occurred during validating the input data due to {}", e),
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
            QueryColumnError::InputInvalidError(e) => write!(f, "Error occurred during validating the input data due to {}", e),
            QueryColumnError::InputInconsistentError(e) => write!(f, "Error occurred during query text build process due to {}", e),
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
            UpdateSetError::InputInvalidError(e) => write!(f, "Error occurred during validating the input data due to {}", e),
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
            InsertValueError::InputInvalidError(e) => write!(f, "Error occurred during validating the input data due to {}", e),
            InsertValueError::InputInconsistentError(e) => write!(f, "Error occurred during check the input data due to {}", e),
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
