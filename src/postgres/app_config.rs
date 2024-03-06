use crate::postgres::validators::validate_alphanumeric_name;

/// Represents the configuration for the application.
///
/// The `AppConfig` struct holds the necessary information for connecting to the database.
pub(crate) struct AppConfig {
    db_username: String,
    db_password: String,
    db_hostname: String,
    db_port: u32,
    db_name: String,
}

impl AppConfig {
    /// Creates a new `AppConfig` instance. This struct is hidden from user in generally.
    ///
    /// This function retrieves the necessary configuration values from environment variables:
    /// - `DB_USER`: The database username.
    /// - `DB_PASSWORD`: The database password.
    /// - `DB_HOST`: The database hostname.
    /// - `DB_PORT`: The database port number.
    /// - `DB_NAME`: The database name.
    ///
    /// `DB_USER` and `DB_PASSWORD`, and `DB_HOST` of the environment variables are missing, an error is returned.
    /// Also, `DB_PORT` and `DB_NAME` are missing, using default value like port=5432 and dbname=postgres.
    ///
    /// # Returns
    ///
    /// - `Ok(AppConfig)`: The `AppConfig` instance with the retrieved configuration values.
    /// - `Err(String)`: An error message indicating which environment variable is missing.
    pub(crate) fn new() -> Result<AppConfig, String> {
        let db_username = match std::env::var("DB_USER") {
            Ok(username) => username,
            Err(_) => return Err("'username' isn't presented by environment variable. Please check your environment.".to_string()),
        };
        let db_password = match std::env::var("DB_PASSWORD") {
            Ok(password) => password,
            Err(_) => return Err("'password' isn't presented by environment variable. Please check your environment.".to_string()),
        };
        let db_hostname = match std::env::var("DB_HOST") {
            Ok(hostname) => hostname,
            Err(_) => return Err("'hostname' isn't presented by environment variable. Please check your environment.".to_string()),
        };
        let db_port = match std::env::var("DB_PORT") {
            Ok(port_number_str) => {
                port_number_str.parse::<u32>().unwrap_or_else(|e| {
                    eprintln!("Port number parse error due to {}", e);
                    5432
                })
            },
            Err(_) => 5432,
        };
        let db_name = match std::env::var("DB_NAME") {
            Ok(dbname) => {
                if !validate_alphanumeric_name(&dbname, "_") {
                    eprintln!("{} is invalid name. 'dbname' is filled as 'postgres' automatically.", dbname);
                    "postgres".to_string()
                } else {
                    dbname
                }
            },
            Err(_) => "postgres".to_string(),
        };

        Ok(Self {
            db_username,
            db_password,
            db_hostname,
            db_port,
            db_name,
        })
    }

    /// Returns a tuple containing references to the values of the database connection parameters.
    ///
    /// # Example
    ///
    /// ```
    /// let config = AppConfig::new();
    /// let (username, password, hostname, port, name) = config.get_values();
    /// println!("Username: {}", username);
    /// println!("Password: {}", password);
    /// println!("Hostname: {}", hostname);
    /// println!("Port: {}", port);
    /// println!("Database name: {}", name);
    /// ```
    ///
    /// # Return
    ///
    /// - `&String` - A reference to the username of the database connection.
    /// - `&String` - A reference to the password of the database connection.
    /// - `&String` - A reference to the hostname of the database connection.
    /// - `u32` - The port number of the database connection.
    /// - `&String` - A reference to the name of the database.
    pub(super) fn get_values(&self) -> (&String, &String, &String, u32, &String) {
        (&self.db_username, &self.db_password, &self.db_hostname, self.db_port, &self.db_name)
    }
}

#[cfg(test)]
mod tests {
    use super::AppConfig;
    use std::env;
    use std::sync::{Arc, Mutex};
    use lazy_static::lazy_static;

    lazy_static! {
        static ref LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
    }

    /// Test if `AppConfig::new` creates an `AppConfig` object correctly when all environment variables are set
    #[test]
    fn test_new_app_config() {
        let _guard = LOCK.lock().unwrap();

        let db_username = "test_user";
        let db_password = "test_password";
        let db_hostname = "localhost";
        let db_port = "10000";
        let db_name = "test_db";

        env::set_var("DB_USER", db_username);
        env::set_var("DB_PASSWORD", db_password);
        env::set_var("DB_HOST", db_hostname);
        env::set_var("DB_PORT", db_port);
        env::set_var("DB_NAME", db_name);

        let config = AppConfig::new().unwrap();

        assert_eq!(config.get_values().0.as_str(), db_username);
        assert_eq!(config.get_values().1.as_str(), db_password);
        assert_eq!(config.get_values().2.as_str(), db_hostname);
        assert_eq!(config.get_values().3, 10000);
        assert_eq!(config.get_values().4.as_str(), db_name);
    }

    /// Test if `AppConfig::new` returns an error when some required environment variables are missing
    #[test]
    fn test_missing_db_config() {
        let _guard = LOCK.lock().unwrap();

        env::remove_var("DB_USER");
        let Err(e) = AppConfig::new() else { panic!() };
        assert_eq!(e, "'username' isn't presented by environment variable. Please check your environment.");
        env::set_var("DB_USER", "username");
        env::remove_var("DB_PASSWORD");
        let Err(e) = AppConfig::new() else { panic!() };
        assert_eq!(e, "'password' isn't presented by environment variable. Please check your environment.");
        env::set_var("DB_PASSWORD", "password");
        env::remove_var("DB_HOST");
        let Err(e) = AppConfig::new() else { panic!() };
        assert_eq!(e, "'hostname' isn't presented by environment variable. Please check your environment.");
        env::set_var("DB_HOST", "localhost");
    }

    /// Test if `AppConfig::new` provides default values when no env vars for `DB_PORT` and `DB_NAME` provided
    #[test]
    fn test_default_values() {
        let _guard = LOCK.lock().unwrap();

        env::remove_var("DB_PORT");
        env::remove_var("DB_NAME");

        let config = AppConfig::new().unwrap();

        assert_eq!(config.db_port, 5432);
        assert_eq!(config.db_name.as_str(), "postgres");
    }
}


