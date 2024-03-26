use std::str::FromStr;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use tokio_postgres::types::ToSql;
use crate::access::converter::{Param, str_to_param};
use crate::access::errors::DataParseError;

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
pub(super) fn box_param_generator(str_params: &[String]) -> Result<Vec<Box<dyn ToSql + Sync>>, DataParseError> {
    let mut params: Vec<Param> = Vec::new();
    for str_param in str_params {
        params.push(str_to_param(str_param)?);
    }

    let mut box_param: Vec<Box<dyn ToSql + Sync>> = Vec::new();

    for param in params {
        match param {
            Param::Int(int) => box_param.push(Box::new(int) as Box<dyn ToSql + Sync>),
            Param::SmallInt(smallint) => box_param.push(Box::new(smallint) as Box<dyn  ToSql + Sync>),
            Param::BigInt(bigint) => box_param.push(Box::new(bigint) as Box<dyn ToSql + Sync>),
            Param::Float(float) => box_param.push(Box::new(float) as Box<dyn ToSql + Sync>),
            Param::Double(double) => box_param.push(Box::new(double) as Box<dyn ToSql + Sync>),
            Param::Decimal(decimal) => box_param.push(Box::new(decimal) as Box<dyn ToSql + Sync>),
            Param::DateTime(datetime) => box_param.push(Box::new(datetime) as Box<dyn ToSql + Sync>),
            Param::Date(date) => box_param.push(Box::new(date) as Box<dyn ToSql + Sync>),
            Param::Time(time) => box_param.push(Box::new(time) as Box<dyn ToSql + Sync>),
            Param::Bool(bool) => box_param.push(Box::new(bool) as Box<dyn ToSql + Sync>),
            Param::Text(text) => box_param.push(Box::new(text) as Box<dyn ToSql + Sync>),
        }
    }
    Ok(box_param)
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
pub(super) fn params_ref_generator<'a>(box_params: &'a[Box<dyn ToSql + Sync>]) -> Vec<&'a(dyn ToSql + Sync)> {
    box_params.iter().map(AsRef::as_ref).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the box_param_generator function with various input cases of different data types.
    /// It checks if the correct boxed values are returned.
    #[test]
    fn test_box_param_generator() {
        let str_params = vec![
            "42".to_string(),
            "hello".to_string(),
            "3.14".to_string(),
            "2023-11-29".to_string(),
            "2023-11-29T21:00:09".to_string(),
            "21:00:09".to_string(),
            "false".to_string()
        ];

        let box_params = box_param_generator(&str_params).unwrap();
        assert_eq!(box_params.len(), str_params.len());
        assert_eq!(
            format!("{:?}", box_params[0]),
            format!("{:?}", Box::new(42) as Box<dyn ToSql + Sync>)
        );
        assert_eq!(
            format!("{:?}", box_params[1]),
            format!("{:?}", Box::new("hello") as Box<dyn ToSql + Sync>)
        );
        assert_eq!(
            format!("{:?}", box_params[2]),
            format!("{:?}", Box::new(3.14) as Box<dyn ToSql + Sync>)
        );
        assert_eq!(
            format!("{:?}", box_params[3]),
            format!("{:?}", Box::new(NaiveDate::from_str("2023-11-29").unwrap()) as Box<dyn ToSql + Sync>)
        );
        assert_eq!(
            format!("{:?}", box_params[4]),
            format!("{:?}", Box::new(NaiveDateTime::from_str("2023-11-29T21:00:09").unwrap()) as Box<dyn ToSql + Sync>)
        );
        assert_eq!(
            format!("{:?}", box_params[5]),
            format!("{:?}", Box::new(NaiveTime::from_str("21:00:09").unwrap()) as Box<dyn ToSql + Sync>)
        );
        assert_eq!(
            format!("{:?}", box_params[6]),
            format!("{:?}", Box::new(false) as Box<dyn ToSql + Sync>));
    }

    /// Tests the params_ref_generator function by using the result of the box_param_generator as input.
    /// It checks if the correct references are returned.
    #[test]
    fn test_params_ref_generator() {
        let str_params = vec![
            "42".to_string(),
            "hello".to_string(),
            "3.14".to_string(),
            "2023-11-29".to_string(),
            "2023-11-29T21:00:09".to_string(),
            "21:00:09".to_string(),
            "false".to_string()
        ];

        let box_params = box_param_generator(&str_params).unwrap();
        let params_ref = params_ref_generator(&box_params);
        assert_eq!(params_ref.len(), str_params.len());
    }
}
