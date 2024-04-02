use std::fmt::{Display, Formatter};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rust_decimal::Decimal;
use crate::generator::base::Generator;
use crate::generator::query::QueryGenerator;

#[derive(Clone)]
pub struct Pair<F> {
    value1: F,
    value2: F,
}

impl<F: Clone + ?Sized> Pair<F> {
    pub fn new(value1: F, value2: F) -> Self {
        Pair {
            value1,
            value2
        }
    }

    pub fn get_values(&self) -> (&F, &F) {
        (&self.value1, &self.value2)
    }

    pub fn get_first(&self) -> &F {
        &self.value1
    }

    pub fn get_second(&self) -> &F {
        &self.value2
    }
}

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
