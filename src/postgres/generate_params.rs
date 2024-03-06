use std::str::FromStr;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use tokio_postgres::types::ToSql;

/// Represents different types of parameters.
///
/// The `Param` enum is used to hold different types of parameters to pass SQL executor.
///
/// # Variants
///
/// - `Text(String)`: A parameter of type `String`.
/// - `Int(i32)`: A parameter of type `i32`.
/// - `Float(f32)`: A parameter of type `f32`.
/// - `Date(NaiveDate)`: A parameter of type `NaiveDate`.
/// - `DateTime(NaiveDateTime)`: A parameter of type `NaiveDateTime`.
/// - `Time(NaiveTime)`: A parameter of type `NaiveTime`.
/// - `Bool(bool)`: A parameter of type `bool`.
enum Param {
    Text(String),
    Int(i32),
    Float(f32),
    Date(NaiveDate),
    DateTime(NaiveDateTime),
    Time(NaiveTime),
    Bool(bool),
}

/// Generates boxed parameters from a vector of strings.
///
/// The function takes a slice of strings `str_params` and attempts to parse each string into
/// different types. If parsing is successful, the parsed value is stored in a `Param` enum variant
/// and added to the `params` vector.
///
/// After parsing all the strings, the function creates a new vector `box_param` and populates it
/// by pushing boxed values of the parsed parameters. Each value is boxed as a trait object
/// implementing the `ToSql` and `Sync` traits.
///
/// # Arguments
///
/// * `str_params` - A reference to a slice of strings to be parsed into parameters.
///
/// # Returns
///
/// A vector of boxed trait objects (`Box<dyn ToSql + Sync>`) containing the parsed parameters.
///
/// # Examples
///
/// ```rust
/// let params = ["1", "Hello", "3.1", "false"]
/// let box_params = box_params_generator(&params)
/// ```
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

/// Generates a new reference to the parameters from a vector of boxed parameters.
///
/// # Arguments
///
/// * `box_params` - A reference to a vector of boxed parameters implementing `ToSql` and `Sync`.
///
/// # Returns
///
/// A new vector containing references to the boxed parameters.
///
/// # Example
///
/// ```rust
/// let box_params: Vec<Box<dyn ToSql + Sync>> = vec![
///     Box::new(42),
///     Box::new("hello"),
///     Box::new(3.14),
/// ];
///
/// let params_refs = params_ref_generator(&box_params);
/// assert_eq!(params_refs.len(), 3);
/// assert_eq!(params_refs[0], &42);
/// assert_eq!(params_refs[1], &"hello");
/// assert_eq!(params_refs[2], &3.14);
/// ```
///
pub(super) fn params_ref_generator<'a>(box_params: &'a[Box<dyn ToSql + Sync>]) -> Vec<&'a(dyn ToSql + Sync)> {
    box_params.iter().map(AsRef::as_ref).collect()
}
