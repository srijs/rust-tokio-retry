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
//! # use futures::Future;
//! # use futures::future::lazy;
//! # use futures03::compat::Future01CompatExt;
//! use tokio_retry::Retry;
//! use tokio_retry::strategy::{ExponentialBackoff, jitter};
//!
//! fn action() -> Result<u64, ()> {
//!     // do some real-world stuff here...
//!     Err(())
//! }
//!
//! #[tokio::main]
//! async fn main() {
//! let retry_strategy = ExponentialBackoff::from_millis(10)
//!     .map(jitter)
//!     .take(3);
//!
//! let future = Retry::spawn(retry_strategy, action).map(|result| {
//!     println!("result {:?}", result);
//! });
//!
//! future.compat().await;
//! }
//! ```

mod action;
mod condition;
mod future;
/// Assorted retry strategies including fixed interval and exponential back-off.
pub mod strategy;

pub use action::Action;
pub use condition::Condition;
pub use future::{Retry, RetryIf};
