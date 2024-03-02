use crate::postgres::postgres_base::PostgresBase;
use crate::postgres::validators::validate_alphanumeric_name;

pub(crate) struct AppConfig {
    pub(crate) db_username: String,
    pub(crate) db_password: String,
    pub(crate) db_hostname: String,
    pub(crate) db_port: u32,
    pub(crate) db_name: String,
}

impl AppConfig {
    pub(crate) fn new() -> Result<AppConfig, String> {
        let db_username = match std::env::var("WORKTIME_DB_USER") {
            Ok(username) => username,
            Err(_) => return Err("'username' isn't presented by environment variable. Please check your environment.".to_string()),
        };
        let db_password = match std::env::var("WORKTIME_DB_PASSWORD") {
            Ok(password) => password,
            Err(_) => return Err("'password' isn't presented by environment variable. Please check your environment.".to_string()),
        };
        let db_hostname = match std::env::var("WORKTIME_DB_HOST") {
            Ok(hostname) => hostname,
            Err(_) => return Err("'hostname' isn't presented by environment variable. Please check your environment.".to_string()),
        };
        let db_port = match std::env::var("WORKTIME_PORT") {
            Ok(port_number_str) => {
                port_number_str.parse::<u32>().unwrap_or_else(|e| {
                    eprintln!("Port number parse error due to {}", e);
                    5432
                })
            },
            Err(_) => 5432,
        };
        let db_name = match std::env::var("WORKTIME_DBNAME") {
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
