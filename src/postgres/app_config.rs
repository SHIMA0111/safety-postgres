use crate::postgres::validators::validate_alphanumeric_name;

/// Represents the configuration for the application.
///
/// The `AppConfig` struct holds the necessary information for connecting to the database.
pub(crate) struct AppConfig {
    pub(crate) db_username: String,
    pub(crate) db_password: String,
    pub(crate) db_hostname: String,
    pub(crate) db_port: u32,
    pub(crate) db_name: String,
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
}
