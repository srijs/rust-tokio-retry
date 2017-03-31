use std::iter::{Iterator, IntoIterator};
use std::time::Duration;
use std::sync::Arc;
use tokio_service::Service;
use tokio_core::reactor::Handle;

use super::{RetryFuture, RetryError, Action};

pub struct ServiceAction<X: Service> {
    inner: Arc<X>,
    request: X::Request
}

impl<X: Service> Action for ServiceAction<X> where X::Request: Clone {
    type Error = X::Error;
    type Item = X::Response;
    type Future = X::Future;

    fn run(&mut self) -> Self::Future {
        self.inner.call(self.request.clone())
    }
}

pub type ServiceRetryFuture<I, X> = RetryFuture<I, ServiceAction<X>>;

/// Middleware that adds retries to a service via a retry strategy.
pub struct RetryService<I, X> {
    inner: Arc<X>,
    handle: Handle,
    strategy: I
}

impl<I: Iterator<Item=Duration>, X> RetryService<I, X> {
    pub fn new<T: IntoIterator<IntoIter=I, Item=Duration>>(handle: Handle, strategy: T, inner: X) -> RetryService<I, X> {
        RetryService{
            inner: Arc::new(inner),
            handle: handle,
            strategy: strategy.into_iter()
        }
    }
}

impl<I: Clone + Iterator<Item=Duration>, X: Service> Service for RetryService<I, X> where X::Request: Clone {
    type Request = X::Request;
    type Response = X::Response;
    type Error = RetryError<X::Error>;
    type Future = ServiceRetryFuture<I, X>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let action = ServiceAction{
            inner: self.inner.clone(),
            request: request
        };

        RetryFuture::spawn(self.handle.clone(), self.strategy.clone(), action)
    }
}
