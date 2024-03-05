use std::str::FromStr;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use tokio_postgres::types::ToSql;

enum Param {
    Text(String),
    Int(i32),
    Float(f32),
    Date(NaiveDate),
    DateTime(NaiveDateTime),
    Time(NaiveTime),
    Bool(bool),
}

pub(super) fn box_param_generator(str_params: &[String]) -> Vec<Box<dyn ToSql + Sync>> {
    let mut params: Vec<Param> = Vec::new();
    for str_param in str_params {
        if let Ok(i) = str_param.parse::<i32>() {
            params.push(Param::Int(i));
        }
        else if let Ok(f) = str_param.parse::<f32>() {
            params.push(Param::Float(f));
        }
        else if let Ok(dt) = NaiveDateTime::from_str(str_param) {
            params.push(Param::DateTime(dt));
        }
        else if let Ok(d) = NaiveDate::from_str(str_param) {
            params.push(Param::Date(d));
        }
        else if let Ok(t) = NaiveTime::from_str(str_param) {
            params.push(Param::Time(t));
        }
        else if let Ok(b) = str_param.parse::<bool>() {
            params.push(Param::Bool(b));
        }
        else {
            params.push(Param::Text(str_param.to_string()))
        }
    }

    let mut box_param: Vec<Box<dyn ToSql + Sync>> = Vec::new();

    for param in params {
        match param {
            Param::Int(i) => box_param.push(Box::new(i) as Box<dyn ToSql + Sync>),
            Param::Float(f) => box_param.push(Box::new(f) as Box<dyn ToSql + Sync>),
            Param::DateTime(dt) => box_param.push(Box::new(dt) as Box<dyn ToSql + Sync>),
            Param::Date(d) => box_param.push(Box::new(d) as Box<dyn ToSql + Sync>),
            Param::Time(t) => box_param.push(Box::new(t) as Box<dyn ToSql + Sync>),
            Param::Bool(b) => box_param.push(Box::new(b) as Box<dyn ToSql + Sync>),
            Param::Text(t) => box_param.push(Box::new(t) as Box<dyn ToSql + Sync>),
        }
    }
    box_param
}

pub(super) fn params_ref_generator<'a>(box_params: &'a[Box<dyn ToSql + Sync>]) -> Vec<&'a(dyn ToSql + Sync)> {
    box_params.iter().map(AsRef::as_ref).collect()
}
