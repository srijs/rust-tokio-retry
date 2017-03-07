use std::time::Duration;
use super::super::RetryStrategy;

/// A retry strategy driven by a fixed interval.
#[derive(Clone)]
pub struct FixedInterval {
    duration: Duration
}

impl FixedInterval {
    /// Constructs a new fixed interval strategy.
    pub fn new(duration: Duration) -> FixedInterval {
        FixedInterval{duration: duration}
    }
}

impl RetryStrategy for FixedInterval {
    fn delay(&mut self) -> Option<Duration> {
        Some(self.duration)
    }
}

#[test]
fn returns_some_fixed() {
    let mut s = FixedInterval::new(Duration::from_millis(123));

    assert_eq!(s.delay(), Some(Duration::from_millis(123)));
    assert_eq!(s.delay(), Some(Duration::from_millis(123)));
    assert_eq!(s.delay(), Some(Duration::from_millis(123)));
}
