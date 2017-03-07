use std::time::Duration;

use super::RetryStrategy;

/// A decorator limiting the number of retries.
#[derive(Clone)]
pub struct LimitedRetries<S: RetryStrategy> {
    inner: S,
    current: usize,
    maximum: usize
}

pub fn limit_retries<S: RetryStrategy>(inner: S, maximum: usize) -> LimitedRetries<S> {
    LimitedRetries{inner: inner, current: 0, maximum: maximum}
}

impl<S: RetryStrategy> RetryStrategy for LimitedRetries<S> {
    fn delay(&mut self) -> Option<Duration> {
        self.current += 1;
        if self.current <= self.maximum {
            self.inner.delay()
        } else {
            None
        }
    }
}

#[test]
fn limits_number_of_retries() {
    use super::super::strategies::FixedInterval;
    let mut s = limit_retries(FixedInterval::new(Duration::from_millis(1000)), 2);

    assert_eq!(s.delay(), Some(Duration::from_millis(1000)));
    assert_eq!(s.delay(), Some(Duration::from_millis(1000)));
    assert_eq!(s.delay(), None);
}
