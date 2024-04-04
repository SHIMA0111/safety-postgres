use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign};
use crate::generator::query::QueryGenerator;
use crate::{Column, Variable};
use crate::utils::errors::GeneratorError;
use crate::utils::helpers::check_aggregation;

pub mod condition;
pub mod join_table;

pub trait MainGenerator {
    fn get_statement(&self) -> String;
    fn get_params(&self) -> Parameters;
    fn get_all_parameters_num(&self) -> u16;
}

pub trait GeneratorPlaceholder {
    fn get_statement(&self, start_placeholder_number: u16) -> String;
    fn get_params(&self) -> Parameters;
    fn get_parameters_number(&self) -> u16;
    fn get_table_name(&self) -> String;
}

pub trait GeneratorPlaceholderWrapper {
    fn get_total_statement(&self, start_placeholder: u16) -> String;
    fn get_all_params(&self) -> Parameters;
    fn len(&self) -> usize;
}

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone, PartialEq)]
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

#[derive(Copy, Clone)]
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
    sort_rules: Vec<SortRule<'a>>,
}

impl <'a> SortRules<'a> {
    pub(crate) fn new() -> SortRules<'a> {
        SortRules { sort_rules: Vec::<SortRule<'a>>::new() }
    }

    pub(crate) fn len(&self) -> usize {
        self.sort_rules.len()
    }

    pub(crate) fn add_sort_rule(&mut self, sort_rule: SortRule<'a>) {
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

pub enum ReferenceValue<'a> {
    Variable(Variable),
    SubQueryAggregation(QueryGenerator<'a>)
}

impl ReferenceValue<'_> {
    pub(crate) fn get_parameter_num(&self) -> u16 {
        match self {
            Self::Variable(_) => 1,
            Self::SubQueryAggregation(value) => value.get_all_parameters_num(),
        }
    }

    pub(crate) fn get_parameters(&self) -> Parameters {
        match self {
            Self::Variable(variable) => Parameters::from(vec![variable.clone()]),
            Self::SubQueryAggregation(query) => query.get_params(),
        }
    }
}

impl From<Variable> for ReferenceValue<'_> {
    fn from(value: Variable) -> Self {
        ReferenceValue::Variable(value)
    }
}

impl <'a> TryFrom<QueryGenerator<'a>> for ReferenceValue<'a> {
    type Error = GeneratorError;

    fn try_from(value: QueryGenerator<'a>) -> Result<Self, Self::Error> {
        let parameter_str = value.get_query_columns();
        let query_columns_vec: Vec<String> =
            parameter_str.split(", ").map(|str| str.to_string()).collect();

        if query_columns_vec.len() != 1 {
            return Err(
                GeneratorError::InconsistentConfigError(
                    format!(
                        "SubQuery for condition value should have only 1 value \
                            but input generator has '{}' columns.", parameter_str)));
        }
        else if !check_aggregation(query_columns_vec[0].clone()) {
            return Err(
                GeneratorError::InconsistentConfigError(
                    format!(
                        "SubQuery for condition value should have only 1 record \
                            so please use aggregation but input is '{}' column",
                        query_columns_vec[0])))
        }
        Ok(ReferenceValue::SubQueryAggregation(value))
    }

}

impl Display for ReferenceValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Variable(value) => write!(f, "{}", value),
            Self::SubQueryAggregation(value) => write!(f, "{}", value.get_statement()),
        }
    }
}

pub struct Parameters {
    parameters: Vec<Variable>,
}

impl Parameters {
    pub fn new() -> Self {
        Self {
            parameters: Vec::new(),
        }
    }

    pub fn push(&mut self, value: Variable) {
        self.parameters.push(value);
    }

    pub fn len(&self) -> usize {
        self.parameters.len()
    }

    pub fn join(&self, delimiter: &str) -> String {
        self.parameters
            .iter()
            .map(|param| format!("{}", param))
            .collect::<Vec<String>>()
            .join(delimiter)
    }
}

impl From<Vec<Variable>> for Parameters {
    fn from(value: Vec<Variable>) -> Self {
        Self {
            parameters: value
        }
    }
}

impl Add for Parameters {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut parameters = self.parameters;
        parameters.extend(rhs.parameters);

        Self { parameters }
    }
}

impl AddAssign for Parameters {
    fn add_assign(&mut self, rhs: Self) {
        self.parameters.extend(rhs.parameters)
    }
}

impl Display for Parameters {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.join(", "))
    }
}
