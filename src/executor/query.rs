use std::error::Error;
use tokio_postgres::Row;
use crate::connector::Connector;
use crate::executor::base::Executor;
use crate::generator::base::Generator;

struct Query {
    connector: Connector,
    result: Vec<Row>
}

impl Executor for Query {
    fn new(connector: Connector) -> Self {
        Self {
            connector,
            result: Vec::<Row>::new()
        }
    }

    async fn execute<T, R, E>(&self, generator: T) -> Result<R, E>
        where
            T: Generator, E: Error
    {
        todo!()
    }
}
