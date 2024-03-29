use std::any::type_name_of_val;
use std::error::Error;
use safety_postgres::connector::connection_config::ConnectionConfig;
use safety_postgres::connector::Connector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = ConnectionConfig::set_config(
        "app", "password", "localhost", 5432, "postgres"
    );

    let connection = Connector::connect(config).await?;
    println!("{:?}", connection);

    println!("{}", type_name_of_val("password"));

    Ok(())
}