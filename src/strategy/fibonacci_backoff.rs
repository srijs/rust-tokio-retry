use std::time::Duration;
use std::iter::Iterator;
use std::u64::{MAX as U64_MAX};

/// A retry strategy driven by the fibonacci series.
///
/// Each retry uses a delay which is the sum of the two previous delays.
///
/// Depending on the problem at hand, a fibonacci retry strategy might
/// perform better and lead to better throughput than the `ExponentialBackoff`
/// strategy.
///
/// See ["A Performance Comparison of Different Backoff Algorithms under Different Rebroadcast Probabilities for MANETs."](http://www.comp.leeds.ac.uk/ukpew09/papers/12.pdf)
/// for more details.
#[derive(Clone)]
pub struct FibonacciBackoff {
    curr: u64,
    next: u64
}

impl FibonacciBackoff {
    /// Constructs a new fibonacci back-off strategy,
    /// given a base duration in milliseconds.
    pub fn from_millis(millis: u64) -> FibonacciBackoff {
        FibonacciBackoff{curr: millis, next: millis}
    }
}

impl Iterator for FibonacciBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        let duration = Duration::from_millis(self.curr);

        if let Some(next_next) = self.curr.checked_add(self.next) {
            self.curr = self.next;
            self.next = next_next;
        } else {
            self.curr = self.next;
            self.next = U64_MAX;
        }

        Some(duration)
    }
}

#[test]
fn returns_the_fibonacci_series_starting_at_10() {
    let mut iter = FibonacciBackoff::from_millis(10);
    assert_eq!(iter.next(), Some(Duration::from_millis(10)));
    assert_eq!(iter.next(), Some(Duration::from_millis(10)));
    assert_eq!(iter.next(), Some(Duration::from_millis(20)));
    assert_eq!(iter.next(), Some(Duration::from_millis(30)));
    assert_eq!(iter.next(), Some(Duration::from_millis(50)));
    assert_eq!(iter.next(), Some(Duration::from_millis(80)));
}

#[test]
fn saturates_at_maximum_value() {
    let mut iter = FibonacciBackoff::from_millis(U64_MAX);
    assert_eq!(iter.next(), Some(Duration::from_millis(U64_MAX)));
    assert_eq!(iter.next(), Some(Duration::from_millis(U64_MAX)));
}