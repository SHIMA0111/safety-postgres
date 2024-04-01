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

pub enum Column<'a> {
    OnMainTable { column_name: &'a str },
    SameSchemaTable { table_name: &'a str, column_name: &'a str },
    AnotherSchemaTable { schema_name: &'a str, table_name: &'a str, column_name: &'a str }
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
