use std::fmt::{Display, Formatter};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rust_decimal::Decimal;

pub struct Pair<F> {
    value1: F,
    value2: F,
}

impl<F> Pair<F> {
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

pub enum Column<'a> {
    OnMainTable { column_name: &'a str },
    SameSchemaTable { table_name: &'a str, column_name: &'a str },
    AnotherSchemaTable { schema_name: &'a str, table_name: &'a str, column_name: &'a str }
}

impl Column<'_> {
    pub(crate) fn get_table_name(&self) -> String {
        match self {
            Column::OnMainTable { .. } => "main".to_string(),
            Column::SameSchemaTable { table_name, .. } => table_name.to_string(),
            Column::AnotherSchemaTable { table_name, .. } => table_name.to_string(),
        }
    }
}

impl Display for Column<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Column::OnMainTable { column_name } => write!(f, "{}", column_name),
            Column::SameSchemaTable { table_name, column_name } => write!(f, "{}.{}", table_name, column_name),
            Column::AnotherSchemaTable { schema_name, table_name, column_name } => write!(f, "{}.{}.{}", schema_name, table_name, column_name),
        }
    }
}
