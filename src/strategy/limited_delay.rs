use std::cmp::min;
use std::time::Duration;

use super::RetryStrategy;

/// A decorator limiting the delay between attempts.
pub struct LimitedDelay<S: RetryStrategy> {
    inner: S,
    maximum: Duration
}

pub fn limit_delay<S: RetryStrategy>(inner: S, maximum: Duration) -> LimitedDelay<S> {
    LimitedDelay{inner: inner, maximum: maximum}
}

impl<S: RetryStrategy> RetryStrategy for LimitedDelay<S> {
    fn delay(&mut self) -> Option<Duration> {
        self.inner.delay().map(|duration| min(duration, self.maximum))
    }
}

#[test]
fn limits_duration() {
    use super::super::strategies::FixedInterval;
    let max = Duration::from_millis(700);
    let mut s = limit_delay(FixedInterval::new(Duration::from_millis(1000)), max);

    assert_eq!(s.delay(), Some(max));
    assert_eq!(s.delay(), Some(max));
    assert_eq!(s.delay(), Some(max));
}
