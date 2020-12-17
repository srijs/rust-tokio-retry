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
//! ## Using the new `tokio` crate
//!
//! ```rust
//! # extern crate futures;
//! # extern crate tokio;
//! # extern crate tokio_retry;
//! #
//! # use futures::Future;
//! # use futures::future::{ready, lazy, FutureExt};
//! use tokio::runtime::Runtime;
//! use tokio_retry::Retry;
//! use tokio_retry::strategy::{ExponentialBackoff, jitter};
//!
//! async fn action() -> Result<u64, ()> {
//!     // do some real-world stuff here...
//!     Err(())
//! }
//!
//! # fn main() {
//! let retry_strategy = ExponentialBackoff::from_millis(10)
//!     .map(jitter)
//!     .take(3);
//!
//! let future = Retry::new(retry_strategy, action).then(|result| {
//!     println!("result {:?}", result);
//!     ready(Ok::<(),()>(()))
//! });
//!
//! let mut rt = Runtime::new().expect("create runtime");
//! rt.block_on(future).expect("retry future");
//! # }
//! ```

mod action;
mod condition;
mod error;
mod future;
/// Assorted retry strategies including fixed interval and exponential back-off.
pub mod strategy;

pub use action::Action;
pub use condition::Condition;
pub use error::Error;
pub use future::{Retry, RetryIf};
