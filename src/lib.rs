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
//! ```rust
//! extern crate futures;
//! extern crate tokio;
//! extern crate tokio_retry;
//!
//! use futures::Future;
//! use futures::future::lazy;
//! use tokio_retry::Retry;
//! use tokio_retry::strategy::{ExponentialBackoff, jitter};
//!
//! fn action() -> Result<u64, ()> {
//!     // do some real-world stuff here...
//!     Ok(42)
//! }
//!
//! fn main() {
//!     tokio::run(lazy(|| {
//!         let retry_strategy = ExponentialBackoff::from_millis(10)
//!             .map(jitter)
//!             .take(3);
//!
//!         Retry::spawn(retry_strategy, action).then(|result| {
//!             println!("result {:?}", result);
//!             Ok(())
//!         })
//!     }));
//! }
//! ```

extern crate futures;
extern crate rand;
extern crate tokio_timer;

mod action;
mod condition;
mod future;
/// Assorted retry strategies including fixed interval and exponential back-off.
pub mod strategy;

pub use action::Action;
pub use condition::Condition;
pub use future::{Error, Retry, RetryIf};
