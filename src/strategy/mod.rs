use std::time::Duration;
use futures::IntoFuture;

use super::future::{retry, Sleep, RetryFuture};

mod jittered;
mod limited_retries;
mod limited_delay;
/// Decorators adding functionality to retry strategies.
pub mod decorators;

/// Trait that specifies a retry behaviour.
pub trait RetryStrategy {
    /// If `Some` is returned, causes a delay of the specified duration before the next attempt.
    /// If `None` is returned, causes no further attempts.
    fn delay(&mut self) -> Option<Duration>;

    /// Introduce full random jitter to the delay between attempts.
    fn jitter(self) -> decorators::Jittered<Self> where Self: Sized {
        jittered::jitter(self)
    }

    /// Limit the number of retries.
    fn limit_retries(self, max_retries: usize) -> decorators::LimitedRetries<Self> where Self: Sized {
        limited_retries::limit_retries(self, max_retries)
    }

    /// Limit the delay between attempts.
    fn limit_delay(self, max_delay: Duration) -> decorators::LimitedDelay<Self> where Self: Sized {
        limited_delay::limit_delay(self, max_delay)
    }

    /// Run the provided action, and if it fails, retry it using this strategy.
    fn run<S, A, F>(self, sleep: S, action: F) -> RetryFuture<S, Self, A, F> where S: Sleep, Self: Sized, A: IntoFuture, F: FnMut() -> A {
        retry(sleep, self, action)
    }
}
