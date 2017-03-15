//! This library provides extensible asynchronous retry behaviours
//! for use with the popular [`futures`](https://crates.io/crates/futures) crate
//! and the ecosystem of [`tokio`](https://tokio.rs/) libraries.
//!
//! # Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tokio-retry = "*"
//! ```
//!
//! By default, `tokio-retry` will work both with the [`Handle`](https://docs.rs/tokio-core/0.1.4/tokio_core/reactor/struct.Handle.html) type from
//! `tokio-core`, and the [`Timer`](https://docs.rs/tokio-timer/0.1.0/tokio_timer/struct.Timer.html) type from `tokio-timer`.
//! Both of these can be disabled or enabled via cargo feature flags:
//!
//! ```toml
//! [dependencies.tokio-retry]
//! version = "*"
//! default-features = false
//! # enable only tokio-core compatibility
//! features = ["tokio_core"]
//! ```
//!
//! # Examples
//!
//! ```rust
//! extern crate futures;
//! extern crate tokio_timer;
//! extern crate tokio_retry;
//!
//! use std::time::Duration;
//! use std::default::Default;
//! use futures::future::Future;
//! use tokio_timer::Timer;
//! use tokio_retry::RetryFuture;
//! use tokio_retry::strategy::{ExponentialBackoff, jitter};
//!
//! fn action() -> Result<u64, ()> {
//!     // do some real-world stuff here...
//!     Ok(42)
//! }
//!
//! pub fn main() {
//!     let retry_strategy = ExponentialBackoff::from_millis(10)
//!         .take(3)
//!         .map(jitter);
//!     let retry_future = RetryFuture::spawn_fn(Timer::default(), retry_strategy, action);
//!     let retry_result = retry_future.wait();
//!
//!     assert_eq!(retry_result, Ok(42));
//! }
//! ```

extern crate either;
extern crate futures;
extern crate rand;
#[cfg(feature = "tokio_core")]
extern crate tokio_core;
#[cfg(feature = "tokio_timer")]
extern crate tokio_timer;
#[cfg(feature = "tokio_service")]
extern crate tokio_service;

mod action;
mod future;
#[cfg(feature = "tokio_service")]
mod middleware;
/// Assorted retry strategies including fixed interval and exponential back-off.
pub mod strategy;

pub use action::{Action, ActionFn};
pub use future::{Sleep, RetryError, RetryFuture};
#[cfg(feature = "tokio_service")]
pub use middleware::{RetryService, ServiceRetryFuture, ServiceAction};
