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
//! tokio-retry = "0.1"
//! ```
//!
//! # Examples
//!
//! ```rust
//! extern crate tokio_core;
//! extern crate tokio_retry;
//!
//! use tokio_core::reactor::Core;
//! use tokio_retry::Retry;
//! use tokio_retry::strategy::{ExponentialBackoff, jitter};
//!
//! fn action() -> Result<u64, ()> {
//!     // do some real-world stuff here...
//!     Ok(42)
//! }
//!
//! fn main() {
//!     let mut core = Core::new().unwrap();
//!
//!     let retry_strategy = ExponentialBackoff::from_millis(10)
//!         .map(jitter)
//!         .take(3);
//!
//!     let retry_future = Retry::spawn(core.handle(), retry_strategy, action);
//!     let retry_result = core.run(retry_future);
//!
//!     assert_eq!(retry_result, Ok(42));
//! }
//! ```

extern crate futures;
extern crate rand;
extern crate tokio_core;
extern crate tokio_service;

mod action;
mod future;
pub mod middleware;
/// Assorted retry strategies including fixed interval and exponential back-off.
pub mod strategy;

pub use action::Action;
pub use future::{Error, Retry};
pub use middleware::Middleware;
