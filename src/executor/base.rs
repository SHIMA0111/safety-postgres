use std::error::Error;
use crate::connector::Connector;
use crate::generator::base::Generator;

pub(super) trait Executor {
    fn new(connection: Connector) -> Self;
    async fn execute<T, R, E>(&self, generator: T) -> Result<R, E>
    where T: Generator, E: Error;
}