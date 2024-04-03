use std::fmt::{Display, Formatter};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rust_decimal::Decimal;
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

/// Represents a column in a database table.
///
/// `Column` is a convenient way to interact with a specific column in a database table.
/// It stores the reference to the parent table and the name of the column.
#[derive(Clone)]
pub struct  Column<'a> {
    table: Table<'a>,
    column_name: &'a str,
}

#[derive(Clone)]
pub enum Table<'a> {
    WithSchema { schema_name: &'a str, table_name: &'a str },
    NonSchema { table_name: &'a str },
    SubQueryAsTable(&'a QueryGenerator<'a>)
}

#[derive(Clone)]
pub struct Schema<'a> {
    schema_name: &'a str,
}
