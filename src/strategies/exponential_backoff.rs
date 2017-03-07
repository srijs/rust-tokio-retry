use std::time::Duration;
use super::super::RetryStrategy;

/// A retry strategy driven by exponential back-off.
///
/// The power corresponds to the number of past attempts.
#[derive(Clone)]
pub struct ExponentialBackoff {
    current: u64,
    base: u64
}

impl ExponentialBackoff {
    /// Constructs a new exponential back-off strategy,
    /// given a base duration in milliseconds.
    ///
    /// The resulting duration is calculated by taking the base to the `n`-th power,
    /// where `n` denotes the number of past attempts.
    pub fn from_millis(base: u64) -> ExponentialBackoff {
        ExponentialBackoff{current: base, base: base}
    }
}

impl RetryStrategy for ExponentialBackoff {
    fn delay(&mut self) -> Option<Duration> {
        let duration = Duration::from_millis(self.current);
        self.current = self.current * self.base;
        return Some(duration);
    }
}

#[test]
fn returns_some_exponential_base_10() {
    let mut s = ExponentialBackoff::from_millis(10);

    assert_eq!(s.delay(), Some(Duration::from_millis(10)));
    assert_eq!(s.delay(), Some(Duration::from_millis(100)));
    assert_eq!(s.delay(), Some(Duration::from_millis(1000)));
}

#[test]
fn returns_some_exponential_base_2() {
    let mut s = ExponentialBackoff::from_millis(2);

    assert_eq!(s.delay(), Some(Duration::from_millis(2)));
    assert_eq!(s.delay(), Some(Duration::from_millis(4)));
    assert_eq!(s.delay(), Some(Duration::from_millis(8)));
}
