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

impl Display for ConditionOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionOperator::Equal => write!(f, "{}", "="),
            ConditionOperator::NotEqual => write!(f, "{}", "!="),
            ConditionOperator::Greater => write!(f, "{}", ">"),
            ConditionOperator::GreaterEq => write!(f, "{}", ">="),
            ConditionOperator::Lower => write!(f, "{}", "<"),
            ConditionOperator::LowerEq => write!(f, "{}", "<="),
            ConditionOperator::In => write!(f, "{}", "IN"),
            ConditionOperator::NotIn => write!(f, "{}", "NOT IN"),
            ConditionOperator::Like => write!(f, "{}", "LIKE"),
            ConditionOperator::NotLike => write!(f, "{}", "NOT LIKE"),
            ConditionOperator::ILike => write!(f, "{}", "ILIKE"),
            ConditionOperator::NotILike => write!(f, "{}", "NOT ILIKE"),
            ConditionOperator::IsNull => write!(f, "{}", "IS NULL"),
            ConditionOperator::IsNotNull => write!(f, "{}", "IS NOT NULL"),
        }
    }
}

#[derive(Clone)]
pub enum BindMethod {
    FirstCondition,
    And,
    Or,
}

impl Display for BindMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BindMethod::And => write!(f, "{}", "AND"),
            BindMethod::Or => write!(f, "{}", "OR"),
            BindMethod::FirstCondition => Ok(())
        }
    }
}

pub enum SortMethod {
    Asc,
    Desc
}

impl Display for SortMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asc => write!(f, "{}", "ASC"),
            Self::Desc => write!(f, "{}", "DESC"),
        }
    }
}

pub enum SortRule<'a> {
    OnMainTable {
        column_name: &'a str,
        sort_method: SortMethod,
    },
    SameSchemaTable {
        table_name: &'a str,
        column_name: &'a str,
        sort_method: SortMethod,
    },
    AnotherSchemaTable {
        schema_name: &'a str,
        table_name: &'a str,
        column_name: &'a str,
        sort_method: SortMethod,
    }
}

impl SortRule<'_> {
    pub(crate) fn get_table_name(&self) -> String {
        match self {
            Self::OnMainTable {..} => "main".to_string(),
            Self::SameSchemaTable {table_name, ..} => table_name.to_string(),
            Self::AnotherSchemaTable {
                schema_name,
                table_name, ..} => format!("{}.{}", schema_name, table_name),
        }
    }

    pub(crate) fn get_sort_statement(&self) -> String {
        match self {
            SortRule::OnMainTable {
                column_name,
                sort_method } => format!("{} {}", column_name, sort_method),
            SortRule::SameSchemaTable {
                table_name,
                column_name,
                sort_method
            } => format!("{}.{} {}", table_name, column_name, sort_method),
            SortRule::AnotherSchemaTable {
                schema_name,
                table_name,
                column_name,
                sort_method
            } => format!("{}.{}.{} {}", schema_name, table_name, column_name, sort_method),
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

impl Aggregation<'_> {
    pub(crate) fn get_table_name(&self) -> String {
        match self {
            Aggregation::Avg(column) => column.get_table_name(),
            Aggregation::Count(column) => column.get_table_name(),
            Aggregation::Sum(column) => column.get_table_name(),
            Aggregation::Min(column) => column.get_table_name(),
            Aggregation::Max(column) => column.get_table_name(),
        }
    }
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
