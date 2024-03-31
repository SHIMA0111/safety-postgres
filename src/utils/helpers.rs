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