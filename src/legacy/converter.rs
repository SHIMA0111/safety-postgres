use std::str::FromStr;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
use itertools::Itertools;
use rust_decimal::Decimal;
use serde_json::{json, Value};
use tokio_postgres::Row;
use crate::legacy::errors::DataParseError;
use crate::legacy::format::{ambiguous_datetime_formats, support_date_formats, support_datetime_formats, support_time_formats, timezone_datetime_formats, unsupported_date_formats, unsupported_datetime_formats, unsupported_time_formats};

const UNSUPPORTED_DATA_TYPE: [&str; 7] = ["f16", "isize", "fsize", "u16", "u32", "u64", "usize"];

/// Represents different types of parameters.
///
/// The `Param` enum is used to hold different types of parameters to pass SQL executor.
///
/// # Variants
///
/// - `Text(String)`: A parameter of type `String`.
/// - `SmallInt(i16)`: A parameter of type `i16`.
/// - `Int(i32)`: A parameter of type `i32`.
/// - `BigInt(i64)`: A parameter of type `i64`.
/// - `Float(f32)`: A parameter of type `f32`.
/// - `Double(f64)`: A parameter of type `f64`.
/// - `Decimal(Decimal)`: A parameter of type `Decimal`.
/// - `Date(NaiveDate)`: A parameter of type `NaiveDate`.
/// - `DateTime(NaiveDateTime)`: A parameter of type `NaiveDateTime`.
/// - `Time(NaiveTime)`: A parameter of type `NaiveTime`.
/// - `Bool(bool)`: A parameter of type `bool`.
pub(super) enum Param {
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

enum ParsedData<T> {
    Parsed(T),
    Text(Param)
}

fn parse_data<T: FromStr>(data: &str) -> ParsedData<T> {
    match data[..data.len() - 3].parse::<T>() {
        Ok(parsed_data) => ParsedData::Parsed(parsed_data),
        Err(_) => ParsedData::Text(Param::Text(data.to_string()))
    }
}

fn parse_datetime_with_zones(data: &str) -> bool {
    if let Ok(_) = DateTime::parse_from_rfc3339(data) {
        return true
    }
    else if let Ok(_) = DateTime::parse_from_rfc2822(data) {
        return true
    }
    else if let Ok(_) = DateTime::parse_from_str(data, "%Y-%m-%d %H:%M:%S %z") {
        return true
    }

    false
}

fn parse_naive_date(data: &str) -> Result<NaiveDate, DataParseError> {
    for support_format in support_date_formats() {
        if let Ok(date) = NaiveDate::parse_from_str(data, support_format.as_str()) {
            let date_chars: Vec<char> = date.to_string().chars().collect();
            if date_chars[..2] == ['0', '0'] {
                continue
            }
            return Ok(date)
        }
    }
    for unsupported_format in unsupported_date_formats() {
        if let Ok(unsupported_date) = NaiveDate::parse_from_str(data, unsupported_format.as_str()) {
            return Err(DataParseError::ParseUnsupportedError(
                format!(
                    "'{}' format is unsupported because the format is ambiguous. \
                    It may be '{}'({}) format but not sure correct or not. \
                    Please use support formats(SEE: https://docs.rs/safety-postgres/latest/safety_postgres ).",
                    data, unsupported_format, unsupported_date)))
        }
    }
    Err(DataParseError::ParseDateTimeError("".to_string()))
}

fn parse_naive_time(data: &str) -> Result<NaiveTime, DataParseError> {
    for support_time_format in support_time_formats() {
        if let Ok(time) = NaiveTime::parse_from_str(data, support_time_format.as_str()) {
            return Ok(time)
        }
    }

    for unsupported_time_format in unsupported_time_formats() {
        if let Ok(_) = NaiveTime::parse_from_str(data, unsupported_time_format.as_str()) {
            return Err(DataParseError::ParseUnsupportedError(
                format!(
                    "'{}' has timezone parameter. Now time with timezone is unsupported. \
                    Please consider to use time without timezone.",
                    data
                )
            ))
        }
    }

    Err(DataParseError::ParseDateTimeError("".to_string()))
}

fn parse_naive_datetime(data: &str) -> Result<NaiveDateTime, DataParseError> {
    for support_format in support_datetime_formats() {
        if let Ok(date) = NaiveDateTime::parse_from_str(data, support_format.as_str()) {
            let date_chars: Vec<char> = date.to_string().chars().collect();
            if date_chars[..2] == ['0', '0'] {
                continue
            }
            return Ok(date)
        }
    }

    for ambiguous_and_timezone_format in unsupported_datetime_formats() {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(data, ambiguous_and_timezone_format.as_str()) {
            return Err(DataParseError::ParseUnsupportedError(
                format!(
                    "'{}' has timezone parameter, and the date part is ambiguous. \
                    It may be '{}'({}) format, but not sure correct or not. \
                    Please modify the format to follow support format the both part of date and time.",
                    data, ambiguous_and_timezone_format, datetime)))
        }
    }

    for ambiguous_datetime_format in ambiguous_datetime_formats() {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(data, ambiguous_datetime_format.as_str()) {
            return Err(DataParseError::ParseUnsupportedError(
                format!(
                    "'{}' format is unsupported because the format is ambiguous at date part. \
                    It may be '{}'({}) format, but not sure correct or not. \
                    Please use support formats at date part\
                    (SEE: https://docs.rs/safety-postgres/latest/safety_postgres ).",
                    data, ambiguous_datetime_format, datetime)))
        }
    }

    for time_with_timezone_format in timezone_datetime_formats() {
        if let Ok(_) = NaiveDateTime::parse_from_str(data, time_with_timezone_format.as_str()) {
            return Err(DataParseError::ParseUnsupportedError(
                format!(
                    "'{}' has timezone parameter. Now time with timezone is unsupported. \
                    Please consider to use time without timezone.",
                    data)))
        }
    }

    Err(DataParseError::ParseDateTimeError("".to_string()))
}

pub(super) fn str_to_param(data: &str) -> Result<Param, DataParseError> {
    let param: Param = if data.ends_with("i16") {
        match parse_data::<i16>(data) {
            ParsedData::Parsed(smallint) => Param::SmallInt(smallint),
            ParsedData::Text(text) => {
                match parse_data::<i64>(data) {
                    ParsedData::Parsed(int) => return Err(DataParseError::ParseIntError(
                        format!("'{}' can not convert to i16(smallint) because overflow the range.", int))),
                    ParsedData::Text(_) => {},
                }
                text
            },
        }
    }
    else if data.ends_with("i64") {
        match parse_data::<i64>(data) {
            ParsedData::Parsed(bigint) => Param::BigInt(bigint),
            ParsedData::Text(text) => text,
        }
    }
    else if data.ends_with("f64") {
        match parse_data::<f64>(data) {
            ParsedData::Parsed(double) => Param::Double(double),
            ParsedData::Text(text) => text,
        }
    }
    else if data.ends_with("dec") {
        match parse_data::<Decimal>(data) {
            ParsedData::Parsed(decimal) => Param::Decimal(decimal),
            ParsedData::Text(text) => text
        }
    }
    else if let Ok(int) = data.parse::<i32>() {
        Param::Int(int)
    }
    else if let Ok(float) = data.parse::<f32>() {
        Param::Float(float)
    }
    else if let Ok(bool) = data.parse::<bool>() {
        Param::Bool(bool)
    }
    else {
        if let Ok(invalid_float) = data.parse::<f64>() {
            return Err(DataParseError::ParseFloatError(
                format!("'{}' can not convert to f32(real) because overflow the range.", invalid_float)))
        }
        else if let Ok(invalid_int) = data.parse::<i64>() {
            return Err(DataParseError::ParseIntError(
                format!("'{}' can not convert to i32(integer) because overflow the range.", invalid_int)))
        }
        else if parse_datetime_with_zones(data) {
            return Err(DataParseError::ParseDateTimeError("DateTime with timezone is unsupported. Please use non timezone datetime instead.".to_string()))
        }
        else if UNSUPPORTED_DATA_TYPE.iter().any(|data_type| data.ends_with(data_type)) {
            let data_chars: Vec<char> = data.chars().collect();
            let data_type = data_chars[data_chars.len() - 3..].iter().collect::<String>();
            return Err(DataParseError::ParseUnsupportedError(format!("[{}]", data_type)))
        }
        match parse_naive_date(data) {
            Ok(date) => Param::Date(date),
            Err(e) => {
                if let DataParseError::ParseUnsupportedError(_) = &e {
                    return Err(e)
                }
                else {
                    match parse_naive_time(data) {
                        Ok(time) => Param::Time(time),
                        Err(e) => {
                            if let DataParseError::ParseUnsupportedError(_) = &e {
                                return Err(e)
                            }
                            else {
                                match parse_naive_datetime(data) {
                                    Ok(datetime) => Param::DateTime(datetime),
                                    Err(e) => {
                                        if let DataParseError::ParseUnsupportedError(_) = &e {
                                            return Err(e)
                                        }
                                        else {
                                            Param::Text(data.to_string())
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    Ok(param)
}

pub(super) fn row_to_value(row: &Row, column: &String) -> Value {
    if let Ok(int) = row.try_get::<&str, i32>(column.as_str()) {
        json!(int)
    }
    else if let Ok(bigint) = row.try_get::<&str, i64>(column.as_str()) {
        json!(bigint)
    }
    else if let Ok(smallint) = row.try_get::<&str, i16>(column.as_str()) {
        json!(smallint)
    }
    else if let Ok(decimal) = row.try_get::<&str, Decimal>(column.as_str()) {
        json!(decimal)
    }
    else if let Ok(float) = row.try_get::<&str, f32>(column.as_str()) {
        json!(float)
    }
    else if let Ok(double) = row.try_get::<&str, f64>(column.as_str()) {
        json!(double)
    }
    else if let Ok(datetime) = row.try_get::<&str, NaiveDateTime>(column.as_str()) {
        json!(datetime.to_string())
    }
    else if let Ok(date) = row.try_get::<&str, NaiveDate>(column.as_str()) {
        json!(date.to_string())
    }
    else if let Ok(time) = row.try_get::<&str, NaiveTime>(column.as_str()) {
        json!(time.to_string())
    }
    else if let Ok(bool) = row.try_get::<&str, bool>(column.as_str()) {
        json!(bool)
    }
    else {
        json!(row.get::<&str, String>(column.as_str()))
    }
}
