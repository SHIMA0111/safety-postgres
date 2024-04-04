use std::error::Error;
use crate::connector::Connector;
use crate::generator::base::MainGenerator;

pub(super) trait Executor {
    fn new(connector: Connector) -> Self;
    async fn execute<T, R, E>(&self, generator: T) -> Result<R, E>
    where
        T: MainGenerator,
        E: Error;
}