use std::iter::{Iterator, IntoIterator};
use std::time::Duration;
use std::sync::Arc;
use tokio_service::Service;
use tokio_core::reactor::Handle;

use super::{Retry, Error};
use super::action::Action;

pub struct ServiceRequest<S: Service> {
    inner: Arc<S>,
    request: S::Request
}

impl<S: Service> Action for ServiceRequest<S> where S::Request: Clone {
    type Error = S::Error;
    type Item = S::Response;
    type Future = S::Future;

    fn run(&mut self) -> Self::Future {
        self.inner.call(self.request.clone())
    }
}

/// Middleware that adds retries to a service via a retry strategy.
pub struct Middleware<I, S> {
    inner: Arc<S>,
    handle: Handle,
    strategy: I
}

impl<I: Iterator<Item=Duration>, S> Middleware<I, S> {
    pub fn new<T: IntoIterator<IntoIter=I, Item=Duration>>(handle: Handle, strategy: T, inner: S) -> Middleware<I, S> {
        Middleware{
            inner: Arc::new(inner),
            handle: handle,
            strategy: strategy.into_iter()
        }
    }
}

impl<I: Clone + Iterator<Item=Duration>, S: Service> Service for Middleware<I, S> where S::Request: Clone {
    type Request = S::Request;
    type Response = S::Response;
    type Error = Error<S::Error>;
    type Future = Retry<I, ServiceRequest<S>>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let action = ServiceRequest{
            inner: self.inner.clone(),
            request: request
        };

        Retry::spawn(self.handle.clone(), self.strategy.clone(), action)
    }
}
