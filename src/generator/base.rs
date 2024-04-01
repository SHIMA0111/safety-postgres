use std::fmt::{Display, Formatter};
use crate::utils::helpers::Column;

pub mod condition;
pub mod join_table;

pub trait Generator {
    fn get_statement(&self) -> String;
    fn get_params(&self) -> Vec<String>;
}

pub enum ConditionOperator {
    Equal,
    NotEqual,
    Greater,
    GreaterEq,
    Lower,
    LowerEq,
    In,
    NotIn,
    Like,
    NotLike,
    ILike,
    NotILike,
    IsNull,
    IsNotNull,
}

impl ConditionOperator {
    pub(crate) fn get_symbol(&self) -> String {
        match self {
            ConditionOperator::Equal => "=".to_string(),
            ConditionOperator::NotEqual => "!=".to_string(),
            ConditionOperator::Greater => ">".to_string(),
            ConditionOperator::GreaterEq => ">=".to_string(),
            ConditionOperator::Lower => "<".to_string(),
            ConditionOperator::LowerEq => "<=".to_string(),
            ConditionOperator::In => "IN".to_string(),
            ConditionOperator::NotIn => "NOT IN".to_string(),
            ConditionOperator::Like => "LIKE".to_string(),
            ConditionOperator::NotLike => "NOT LIKE".to_string(),
            ConditionOperator::ILike => "ILIKE".to_string(),
            ConditionOperator::NotILike => "NOT ILIKE".to_string(),
            ConditionOperator::IsNull => "IS NULL".to_string(),
            ConditionOperator::IsNotNull => "IS NOT NULL".to_string(),
        }
    }
}

pub enum ChainMethod {
    And,
    Or
}

impl ChainMethod {
    pub(crate) fn get_string(&self) -> String {
        match self {
            ChainMethod::And => "AND".to_string(),
            ChainMethod::Or => "OR".to_string(),
        }
    }
}

pub enum SortMethod {
    Asc,
    Desc
}

pub struct SortRule<'a> {
    table_name: &'a str,
    schema_name: Option<&'a str>,
    column_name: &'a str,
    sort_method: SortMethod,
}

impl SortRule<'_> {
    pub fn get_table_name(&self) -> String {
        match self.schema_name {
            Some(schema) => format!("{}.{}", schema, self.table_name),
            None => format!("{}", self.table_name),
        }
    }
}

pub enum Aggregation<'a> {
    Avg(Column<'a>),
    Count(Column<'a>),
    Sum(Column<'a>),
    Min(Column<'a>),
    Max(Column<'a>),
}

impl Display for Aggregation<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Aggregation::Avg(column) => write!(f, "AVG({})", column),
            Aggregation::Count(column) => write!(f, "COUNT({})", column),
            Aggregation::Sum(column) => write!(f, "SUM({})", column),
            Aggregation::Min(column) => write!(f, "MIN({})", column),
            Aggregation::Max(column) => write!(f, "MAX({})", column),
        }
    }
}
