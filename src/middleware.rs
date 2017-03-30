use std::iter::{Iterator, IntoIterator};
use std::time::Duration;
use std::sync::Arc;
use futures::Future;
use tokio_service::Service;

use super::{Sleep, RetryFuture, RetryError, Action};

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

pub type ServiceRetryFuture<S, I, X> = RetryFuture<S, I, ServiceAction<X>>;

/// Middleware that adds retries to a service via a retry strategy.
pub struct RetryService<S, I, X> {
    inner: Arc<X>,
    sleep: S,
    strategy: I
}

impl<S: Sleep, I: Iterator<Item=Duration>, X> RetryService<S, I, X> {
    pub fn new<T: IntoIterator<IntoIter=I, Item=Duration>>(sleep: S, strategy: T, inner: X) -> RetryService<S, I, X> {
        RetryService{
            inner: Arc::new(inner),
            sleep: sleep,
            strategy: strategy.into_iter()
        }
    }
}

impl<S: Clone + Sleep, I: Clone + Iterator<Item=Duration>, X: Service> Service for RetryService<S, I, X> where X::Request: Clone {
    type Request = X::Request;
    type Response = X::Response;
    type Error = RetryError<X::Error, <S::Future as Future>::Error>;
    type Future = ServiceRetryFuture<S, I, X>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let action = ServiceAction{
            inner: self.inner.clone(),
            request: request
        };

        RetryFuture::spawn(self.sleep.clone(), self.strategy.clone(), action)
    }
}
