use std::any::{type_name, type_name_of_val};
use std::error::Error;
use std::str::FromStr;
use serde_json::from_str;
use crate::utils::errors::ConnectionConfigError;


pub(super) struct ConnectionConfig {
    username: String,
    password: String,
    hostname: String,
    port: u32,
    database_name: String,
}

impl ConnectionConfig {
    pub fn config_from_env() -> Self {
        
    }

    fn config_getter<T: ?Sized + FromStr>(config_name: &str) -> Result<T, ConnectionConfigError> {
        match std::env::var(config_name) {
            Ok(value) => {
                if let Ok(parsed_value) = value.parse::<T>() {
                    Ok(parsed_value)
                }
                else {
                    return Err(ConnectionConfigError::TypeError(
                        format!("'{}'(value: {}) can't be parsed to '{}'",
                                config_name, value, type_name::<T>())
                    ))
                }
            },
            Err(_) => {
                return Err(ConnectionConfigError::UndefinedValueError(
                    format!("'{}' is undefined on your environment.", config_name)
                ))
            }
        }
    }
}