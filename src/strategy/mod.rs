use std::time::Duration;
use futures::IntoFuture;
use tokio_timer::Timer;

use super::future::{retry, RetryFuture};

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
    fn run<A, F>(self, timer: Timer, action: F) -> RetryFuture<Self, A, F> where Self: Sized, A: IntoFuture, F: FnMut() -> A {
        retry(self, timer, action)
    }
}
