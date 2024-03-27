mod connection_config;

use tokio_postgres::Client;
use crate::connector::connection_config::ConnectionConfig;

pub(crate) struct Connector {
    config: ConnectionConfig,
    client: Option<Client>
}


