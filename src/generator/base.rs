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

#[derive(Clone, PartialEq)]
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

pub(crate) struct SortRules<'a> {
    sort_rules: Vec<&'a SortRule<'a>>,
}

impl <'a> SortRules<'a> {
    pub(crate) fn new() -> SortRules<'a> {
        SortRules { sort_rules: Vec::<&'a SortRule<'a>>::new() }
    }

    pub(crate) fn len(&self) -> usize {
        self.sort_rules.len()
    }

    pub(crate) fn add_sort_rule(&mut self, sort_rule: &'a SortRule<'a>) {
        self.sort_rules.push(sort_rule);
    }

    pub(crate) fn get_sort_rule_statement(&self) -> String {
        let sort_columns =  self.sort_rules
            .iter()
            .map(|sort_rule| sort_rule.get_sort_statement())
            .collect::<Vec<String>>()
            .join(", ");

        format!("ORDER BY {}", sort_columns)
    }
}

pub struct  SortRule<'a> {
    column: &'a Column<'a>,
    sort_method: SortMethod,
}

impl <'a> SortRule<'a> {
    pub fn new(column: &'a Column<'a>, sort_method: SortMethod) -> SortRule<'a> {
        Self {
            column,
            sort_method
        }
    }

    pub(crate) fn get_table_name(&self) -> String {
        self.column.get_table_name()
    }

    pub(crate) fn get_sort_statement(&self) -> String {
        format!("{} {}", self.column, self.sort_method)
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
