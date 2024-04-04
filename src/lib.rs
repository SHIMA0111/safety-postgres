use std::fmt::{Display, Formatter};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rust_decimal::Decimal;
use crate::generator::base::{MainGenerator, Parameters};
use crate::generator::query::QueryGenerator;

pub mod legacy;
pub mod connector;
pub mod utils;
pub mod generator;
mod converter;
mod executor;

/// Represents a variable that can hold different types of values.
///
/// The `Variable` enum is used to store values of different types. Each variant of the enum
/// corresponds to a specific type of value that can be stored.
///
/// # Variants
///
/// - `Text(String)`: Represents a variable that holds a text value.
/// - `SmallInt(i16)`: Represents a variable that holds a small integer value (-32,768 to 32,767).
/// - `Int(i32)`: Represents a variable that holds an integer value (-2,147,483,648 to 2,147,483,647).
/// - `BigInt(i64)`: Represents a variable that holds a big integer value (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807).
/// - `Float(f32)`: Represents a variable that holds a single-precision floating-point value.
/// - `Double(f64)`: Represents a variable that holds a double-precision floating-point value.
/// - `Decimal(Decimal)`: Represents a variable that holds a decimal value.
/// - `Date(NaiveDate)`: Represents a variable that holds a date value.
/// - `DateTime(NaiveDateTime)`: Represents a variable that holds a date and time value.
/// - `Time(NaiveTime)`: Represents a variable that holds a time value.
/// - `Bool(bool)`: Represents a variable that holds a boolean value.
#[derive(Clone)]
pub enum Variable {
    Text(String),
    SmallInt(i16),
    Int(i32),
    BigInt(i64),
    Float(f32),
    Double(f64),
    Decimal(Decimal),
    Date(NaiveDate),
    DateTime(NaiveDateTime),
    Time(NaiveTime),
    Bool(bool),
}

impl From<String> for Variable {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<i16> for Variable {
    fn from(value: i16) -> Self {
        Self::SmallInt(value)
    }
}

impl From<i32> for Variable {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<i64> for Variable {
    fn from(value: i64) -> Self {
        Self::BigInt(value)
    }
}

impl From<f32> for Variable {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<f64> for Variable {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<Decimal> for Variable {
    fn from(value: Decimal) -> Self {
        Self::Decimal(value)
    }
}

impl From<NaiveDate> for Variable {
    fn from(value: NaiveDate) -> Self {
        Self::Date(value)
    }
}

impl From<NaiveDateTime> for Variable {
    fn from(value: NaiveDateTime) -> Self {
        Self::DateTime(value)
    }
}

impl From<NaiveTime> for Variable {
    fn from(value: NaiveTime) -> Self {
        Self::Time(value)
    }
}

impl From<bool> for Variable {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::Text(value) => write!(f, "{}", value),
            Variable::SmallInt(value) => write!(f, "{}", value),
            Variable::Int(value) => write!(f, "{}", value),
            Variable::BigInt(value) => write!(f, "{}", value),
            Variable::Float(value) => write!(f, "{}", value),
            Variable::Double(value) => write!(f, "{}", value),
            Variable::Decimal(value) => write!(f, "{}", value),
            Variable::Date(value) => write!(f, "{}", value),
            Variable::DateTime(value) => write!(f, "{}", value),
            Variable::Time(value) => write!(f, "{}", value),
            Variable::Bool(value) => write!(f, "{}", value),
        }
    }
}

/// Represents a column in a database table.
///
/// `Column` is a convenient way to interact with a specific column in a database table.
/// It stores the reference to the parent table and the name of the column.
#[derive(Clone)]
pub struct  Column<'a> {
    table: Table<'a>,
    column_name: &'a str,
}

impl <'a> Column<'a> {
    pub fn create_column(schema_name: Option<&'a str>, table_name: &'a str, column_name: &'a str) -> Column<'a> {
        let table = match schema_name {
            Some(schema) => Table::WithSchema { schema_name: schema, table_name },
            None => Table::NonSchema { table_name }
        };

        Self {
            table,
            column_name,
        }
    }
    pub fn create_sub_query_column(query: &'a QueryGenerator<'a>, column_name: &'a str) -> Column<'a> {
        Self {
            table: Table::SubQueryAsTable(query),
            column_name,
        }
    }

    pub(crate) fn get_table_name(&self) -> String {
        self.table.get_table_name()
    }

    pub(crate) fn get_parameter_num(&self) -> u16 {
        self.table.get_parameter_num()
    }

    fn create_column_by_table(table: &'a Table<'a>, column_name: &'a str) -> Column<'a> {
        Self {
            table: table.clone(),
            column_name,
        }
    }
}

impl Display for Column<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.table.get_table_name(), self.column_name)
    }
}

#[derive(Clone)]
pub enum Table<'a> {
    WithSchema { schema_name: &'a str, table_name: &'a str },
    NonSchema { table_name: &'a str },
    SubQueryAsTable(&'a QueryGenerator<'a>)
}

impl <'a> Table<'a> {
    pub fn create_table(schema_name: Option<&'a str>, table_name: &'a str) -> Table<'a> {
        match schema_name {
            Some(schema) => Self::WithSchema { schema_name: schema, table_name },
            None => Self::NonSchema { table_name },
        }
    }

    pub fn create_sub_query_table(query: &'a QueryGenerator<'a>) -> Table<'a> {
        Table::SubQueryAsTable(query)
    }

    pub fn get_column(&'a self, column_name: &'a str) -> Column<'a> {
        Column::create_column_by_table(&self, column_name)
    }

    pub(crate) fn get_schema_name(&self) -> Option<String> {
        match self {
            Self::WithSchema { schema_name, .. } => Some(format!("{}", schema_name)),
            Self::NonSchema { .. } => None,
            Self::SubQueryAsTable(_) => None,
        }
    }

    pub(crate) fn get_table_name(&self) -> String {
        match self {
            Table::WithSchema {
                schema_name,
                table_name } => format!("{}.{}", schema_name, table_name),
            Table::NonSchema { table_name } => format!("{}", table_name),
            Table::SubQueryAsTable(_) => "sub_query".to_string()
        }
    }

    pub(crate) fn get_parameter_num(&self) -> u16 {
        match self {
            Self::NonSchema { .. } | Self::WithSchema { .. } => 0,
            Self::SubQueryAsTable(query) => query.get_all_parameters_num()
        }
    }

    pub(crate) fn get_parameters(&self) -> Parameters {
        match self {
            Self::WithSchema {..} | Self::NonSchema { .. } => Parameters::new(),
            Self::SubQueryAsTable(query) => query.get_params()
        }
    }
}

impl Display for Table<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Table::WithSchema {
                schema_name,
                table_name } => write!(f, "{}.{}", schema_name, table_name),
            Table::NonSchema { table_name } => write!(f, "{}", table_name),
            Table::SubQueryAsTable(query_generator) => write!(f, "({}) AS sub_query", query_generator.get_statement()),
        }
    }
}


#[derive(Clone)]
pub struct Schema<'a> {
    schema_name: &'a str,
}

impl <'a> Schema<'a> {
    pub fn new(schema_name: &'a str) -> Schema<'a> {
        Self {
            schema_name,
        }
    }

    pub fn get_table(&self, table_name: &'a str) -> Table<'a> {
        Table::create_table(Some(self.schema_name), table_name)
    }

    pub fn get_column(&self, table_name: &'a str, column_name: &'a str) -> Column<'a> {
        Column::create_column(Some(self.schema_name), table_name, column_name)
    }
}

impl Display for Schema<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.schema_name)
    }
}

