//! This library provides extensible asynchronous retry behaviours
//! for use with the ecosystem of [`tokio`](https://tokio.rs/) libraries.
//!
//! # Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tokio-retry = "0.3"
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! # extern crate tokio;
//! # extern crate tokio_retry;
//! #
//! use tokio_retry::Retry;
//! use tokio_retry::strategy::ExponentialBackoff;
//!
//! async fn action() -> Result<u64, ()> {
//!     // do some real-world stuff here...
//!     Err(())
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), ()> {
//! let retry_strategy = ExponentialBackoff::from_millis(10)
//!     .take(3);    // limit to 3 retries
//!
//! let result = Retry::spawn(retry_strategy, action).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//! [jitter]
//!
//! To use jitter, add this to your Cargo.toml
//!
//! ```toml
//! [dependencies]
//! tokio-retry = { version = "0.3", features = ["jitter"] }
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! # extern crate tokio;
//! # extern crate tokio_retry;
//! #
//! use tokio_retry::Retry;
//! use tokio_retry::strategy::{ExponentialBackoff, jitter};
//!
//! let retry_strategy = ExponentialBackoff::from_millis(10)
//!    .map(jitter) // add jitter to the retry interval
//!    .take(3);    // limit to 3 retries
//!
//!
//!

#![allow(warnings)]

mod action;
mod condition;
mod future;
/// Assorted retry strategies including fixed interval and exponential back-off.
pub mod strategy;

pub use action::Action;
pub use condition::Condition;
pub use future::{Retry, RetryIf};
