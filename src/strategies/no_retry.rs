use std::time::Duration;
use super::super::RetryStrategy;

/// A retry strategy that will not attempt any retries.
pub struct NoRetry {}

impl RetryStrategy for NoRetry {
    fn delay(&mut self) -> Option<Duration> {
        None
    }
}

#[test]
fn returns_none() {
    let mut s = NoRetry{};

    assert_eq!(s.delay(), None);
}
