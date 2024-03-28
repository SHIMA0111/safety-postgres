use std::any::type_name;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::utils::errors::ConnectionConfigError;


pub struct ConnectionConfig {
    username: String,
    password: String,
    hostname: String,
    port: u16,
    database_name: String,
}

impl ConnectionConfig {
    pub fn config_from_env() -> Result<Self, ConnectionConfigError> {
        let username = Self::config_getter::<String>("DB_USER")?;
        let password = Self::config_getter::<String>("DB_PASSWORD")?;
        let hostname = Self::config_getter::<String>("DB_HOST")?;

        let port = Self::config_getter_with_default::<u16>("DB_PORT", 5432)?;
        let database_name = Self::config_getter_with_default::<String>("DB_NAME", "postgres".to_string())?;

        Ok(Self { username, password, hostname, port, database_name })
    }

    pub fn set_config(
        username: &str,
        password: &str,
        hostname: &str,
        port: u16,
        database_name: &str) -> Self {
        Self {
                username: username.to_string(),
                password: password.to_string(),
                hostname: hostname.to_string(),
                port,
                database_name: database_name.to_string(),
        }
    }

    pub(crate) fn get_user(&self) -> &str {
        self.username.as_str()
    }

    pub(crate) fn get_password(&self) -> &str {
        self.password.as_str()
    }

    pub(crate) fn get_hostname(&self) -> &str {
        self.hostname.as_str()
    }

    pub(crate) fn get_port(&self) -> u16 {
        self.port
    }

    pub(crate) fn get_db_name(&self) -> &str {
        self.database_name.as_str()
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

    fn config_getter_with_default<T: ?Sized + FromStr>(config_name: &str, default_value: T) -> Result<T, ConnectionConfigError> {
        let value = match Self::config_getter::<T>(config_name) {
            Ok(get_value) => get_value,
            Err(e) => {
                if let ConnectionConfigError::UndefinedValueError(_) = e {
                    default_value
                }
                else {
                    return Err(e)
                }
            }
        };

        Ok(value)
    }
}

impl Display for ConnectionConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "psql://{}:****@{}:{}/{}", self.username, self.hostname, self.port, self.database_name)
    }
}
