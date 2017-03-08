use std::time::Duration;
use std::iter::{Iterator, IntoIterator};
use super::super::RetryStrategy;

/// A retry strategy backed by an iterator.
///
/// Example:
///
/// ```
/// use std::time::Duration;
/// use tokio_retry::strategies::FromIterator;
///
/// let delays = vec![
///   Duration::from_millis(10),
///   Duration::from_millis(20),
///   Duration::from_millis(30)
/// ];
///
/// let retry = FromIterator::new(delays);
/// ```
#[derive(Clone)]
pub struct FromIterator<T> where T: IntoIterator<Item=Duration> {
    iter: T::IntoIter
}

impl<T> FromIterator<T> where T: IntoIterator<Item=Duration> {
    pub fn new(iter: T) -> FromIterator<T> {
        FromIterator{iter: iter.into_iter()}
    }
}

impl<T> RetryStrategy for FromIterator<T> where T: IntoIterator<Item=Duration> {
    fn delay(&mut self) -> Option<Duration> {
        self.iter.next()
    }
}

#[test]
fn returns_from_iterator() {
    let delays = vec![
        Duration::from_millis(10),
        Duration::from_millis(20),
        Duration::from_millis(30)
    ];
    let mut s = FromIterator::new(delays);

    assert_eq!(s.delay(), Some(Duration::from_millis(10)));
    assert_eq!(s.delay(), Some(Duration::from_millis(20)));
    assert_eq!(s.delay(), Some(Duration::from_millis(30)));
    assert_eq!(s.delay(), None);
}
