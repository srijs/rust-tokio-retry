//! Middleware for tokio services that adds automatic retries
//! in case of failure.
//!
//! # Examples
//!
//! ```rust
//! extern crate futures;
//! extern crate tokio_core;
//! extern crate tokio_service;
//! extern crate tokio_retry;
//!
//! use std::io;
//!
//! use futures::{BoxFuture, Future, future};
//! use tokio_core::reactor::Core;
//! use tokio_service::Service;
//! use tokio_retry::Middleware;
//! use tokio_retry::strategy::{ExponentialBackoff, jitter};
//!
//! struct EchoService;
//!
//! impl Service for EchoService {
//!    type Request = String;
//!    type Response = String;
//!    type Error = ();
//!    type Future = BoxFuture<String, ()>;
//!    fn call(&self, input: String) -> Self::Future {
//!        future::ok(input).boxed()
//!    }
//! }
//!
//! fn main() {
//!     let mut core = Core::new().unwrap();
//!
//!     let retry_strategy = || ExponentialBackoff::from_millis(10)
//!         .map(jitter)
//!         .take(3);
//!
//!     let retry_service = Middleware::new(core.handle(), retry_strategy, EchoService);
//!     let retry_result = core.run(retry_service.call("hello world!".to_string()));
//!
//!     assert_eq!(retry_result, Ok("hello world!".to_string()));
//! }
//! ```

use std::iter::{Iterator, IntoIterator};
use std::time::Duration;
use std::sync::Arc;
use tokio_service::Service;
use tokio_core::reactor::Handle;

use super::{Retry, Error};
use super::action::Action;

/// Represents a retryable request to a service.
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
pub struct Middleware<T, S> {
    inner: Arc<S>,
    handle: Handle,
    strategy: T
}

/// Trait to produce iterators that will be used as retry strategies.
///
/// Can be implemented directly, but the simplest way to instantiate
/// a strategy factory is by leveraging the `impl` for `Fn()`:
///
/// ```rust
/// # use tokio_retry::strategy::ExponentialBackoff;
/// let retry_strategy = || ExponentialBackoff::from_millis(10);
/// ```
pub trait StrategyFactory {
    type Iter: Iterator<Item=Duration>;

    fn get_strategy(&self) -> Self::Iter;
}

impl<F, I: IntoIterator<Item=Duration>> StrategyFactory for F where F: Fn() -> I {
    type Iter = I::IntoIter;

    fn get_strategy(&self) -> Self::Iter {
        self().into_iter()
    }
}

impl<T: StrategyFactory, S> Middleware<T, S> {
    pub fn new(handle: Handle, strategy: T, inner: S) -> Middleware<T, S> {
        Middleware{
            inner: Arc::new(inner),
            handle: handle,
            strategy: strategy
        }
    }
}

impl<T: StrategyFactory, S: Service> Service for Middleware<T, S> where S::Request: Clone {
    type Request = S::Request;
    type Response = S::Response;
    type Error = Error<S::Error>;
    type Future = Retry<T::Iter, ServiceRequest<S>>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let action = ServiceRequest{
            inner: self.inner.clone(),
            request: request
        };

        Retry::spawn(self.handle.clone(), self.strategy.get_strategy(), action)
    }
}
