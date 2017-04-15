use std::time::Duration;
use std::iter::Iterator;
use std::u64::MAX as U64_MAX;

/// A retry strategy driven by exponential back-off.
///
/// The power corresponds to the number of past attempts.
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    current: u64,
    base: u64,
    factor: u64,
}

impl ExponentialBackoff {
    /// Constructs a new exponential back-off strategy,
    /// given a base duration in milliseconds.
    ///
    /// The resulting duration is calculated by taking the base to the `n`-th power,
    /// where `n` denotes the number of past attempts.
    pub fn from_millis(base: u64) -> ExponentialBackoff {
        ExponentialBackoff {
            current: base,
            base: base,
            factor: 1u64,
        }
    }

    /// A multiplicative factor that will be applied to the retry delay.
    ///
    /// For example, using a factor of `1000` will make each delay in units of seconds.
    ///
    /// Default factor is `1`.
    pub fn factor(mut self, factor: u64) -> ExponentialBackoff {
        self.factor = factor;
        self
    }
}

impl Iterator for ExponentialBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        // set delay duration by applying factor
        let duration = if let Some(duration) = self.current.checked_mul(self.factor) {
            Duration::from_millis(duration)
        } else {
            Duration::from_millis(U64_MAX)
        };

        if let Some(next) = self.current.checked_mul(self.base) {
            self.current = next;
        } else {
            self.current = U64_MAX;
        }

        Some(duration)
    }
}

#[test]
fn returns_some_exponential_base_10() {
    let mut s = ExponentialBackoff::from_millis(10);

    assert_eq!(s.next(), Some(Duration::from_millis(10)));
    assert_eq!(s.next(), Some(Duration::from_millis(100)));
    assert_eq!(s.next(), Some(Duration::from_millis(1000)));
}

#[test]
fn returns_some_exponential_base_2() {
    let mut s = ExponentialBackoff::from_millis(2);

    assert_eq!(s.next(), Some(Duration::from_millis(2)));
    assert_eq!(s.next(), Some(Duration::from_millis(4)));
    assert_eq!(s.next(), Some(Duration::from_millis(8)));
}

#[test]
fn saturates_at_maximum_value() {
    let mut s = ExponentialBackoff::from_millis(U64_MAX - 1);

    assert_eq!(s.next(), Some(Duration::from_millis(U64_MAX - 1)));
    assert_eq!(s.next(), Some(Duration::from_millis(U64_MAX)));
    assert_eq!(s.next(), Some(Duration::from_millis(U64_MAX)));
}

#[test]
fn can_use_factor_to_get_seconds() {
    let factor = 1000;
    let mut s = ExponentialBackoff::from_millis(2).factor(factor);

    assert_eq!(s.next(), Some(Duration::from_secs(2)));
    assert_eq!(s.next(), Some(Duration::from_secs(4)));
    assert_eq!(s.next(), Some(Duration::from_secs(8)));
}
