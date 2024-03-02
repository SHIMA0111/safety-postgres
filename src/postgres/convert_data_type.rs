use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use tokio_postgres::types::Date;

enum Param<Tz: TimeZone> {
    Text(String),
    Int(i32),
    Float(f32),
    Date(NaiveDate),
    DateWithTimezone(Date<Tz>),
    DateTime(NaiveDateTime),
    DateTimeWithTimezone(DateTime<Tz>),
    Time(NaiveTime),
    Bool(bool),
}