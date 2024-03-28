pub mod connection_config;

use std::fmt::{Debug, Formatter};
use tokio_postgres::{Client, NoTls, Error as PGError};
use crate::connector::connection_config::ConnectionConfig;

pub struct Connector {
    config: ConnectionConfig,
    client: Option<Client>,
}

impl Connector {
    pub async fn connect(config: ConnectionConfig) -> Result<Self, PGError> {
        let (client, connection) = tokio_postgres::Config::new()
            .user(config.get_user())
            .password(config.get_password())
            .host(config.get_hostname())
            .port(config.get_port())
            .dbname(config.get_db_name())
            .connect(NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection failed due to {}", e);
            }
        });
        Ok(Self {
            config,
            client: Some(client)
        })
    }
}

impl Debug for Connector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Connection Established to {}!!", self.config)
    }
}
