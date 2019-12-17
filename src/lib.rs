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
//! tokio-retry = "0.2"
//! ```
//!
//! # Examples
//!
//! ## Using the `tokio` crate
//!
//! ```rust
//! # extern crate tokio;
//! # extern crate tokio_retry;
//! # extern crate futures;
//! #
//! # use std::future::Future;
//! # use futures::prelude::*;
//! use tokio::runtime::Runtime;
//! use tokio_retry::Retry;
//! use tokio_retry::strategy::{ExponentialBackoff, jitter};
//!
//! fn action() -> impl Future<Output=Result<u64, ()>> {
//!     // do some real-world stuff here...
//!     futures::future::err(())
//! }
//!
//! # fn main() {
//! let mut rt = Runtime::new().unwrap();
//!
//! let retry_strategy = ExponentialBackoff::from_millis(10)
//!     .map(jitter)
//!     .take(3);
//!
//! let future = Retry::spawn(retry_strategy, action).then(|result| {
//!     println!("result {:?}", result);
//!     futures::future::ok::<_, ()>(())
//! });
//!
//! rt.block_on(future);
//! # }
//! ```

mod action;
mod condition;
mod future;
/// Assorted retry strategies including fixed interval and exponential back-off.
pub mod strategy;

pub use action::Action;
pub use condition::Condition;
pub use future::{Retry, RetryIf};
