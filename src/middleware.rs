use std::iter::{Iterator, IntoIterator};
use std::time::Duration;
use std::sync::Arc;
use tokio_service::Service;
use tokio_core::reactor::Handle;

use super::{RetryFuture, RetryError, Action};

pub struct ServiceAction<S: Service> {
    inner: Arc<S>,
    request: S::Request
}

impl<S: Service> Action for ServiceAction<S> where S::Request: Clone {
    type Error = S::Error;
    type Item = S::Response;
    type Future = S::Future;

    fn run(&mut self) -> Self::Future {
        self.inner.call(self.request.clone())
    }
}

pub type ServiceRetryFuture<I, S> = RetryFuture<I, ServiceAction<S>>;

/// Middleware that adds retries to a service via a retry strategy.
pub struct RetryService<I, S> {
    inner: Arc<S>,
    handle: Handle,
    strategy: I
}

impl<I: Iterator<Item=Duration>, S> RetryService<I, S> {
    pub fn new<T: IntoIterator<IntoIter=I, Item=Duration>>(handle: Handle, strategy: T, inner: S) -> RetryService<I, S> {
        RetryService{
            inner: Arc::new(inner),
            handle: handle,
            strategy: strategy.into_iter()
        }
    }
}

impl<I: Clone + Iterator<Item=Duration>, S: Service> Service for RetryService<I, S> where S::Request: Clone {
    type Request = S::Request;
    type Response = S::Response;
    type Error = RetryError<S::Error>;
    type Future = ServiceRetryFuture<I, S>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let action = ServiceAction{
            inner: self.inner.clone(),
            request: request
        };

        RetryFuture::spawn(self.handle.clone(), self.strategy.clone(), action)
    }
}
