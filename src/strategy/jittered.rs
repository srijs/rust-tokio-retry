use std::time::Duration;
use rand::{random, Closed01};

use super::RetryStrategy;

/// A decorator adding full random jitter to a retry strategy.
#[derive(Clone)]
pub struct Jittered<S: RetryStrategy> {
    inner: S
}

pub fn jitter<S: RetryStrategy>(inner: S) -> Jittered<S> {
    Jittered{inner: inner}
}

impl<S: RetryStrategy> RetryStrategy for Jittered<S> {
    fn delay(&mut self) -> Option<Duration> {
        self.inner.delay().map(|duration| {
            let Closed01(jitter) = random::<Closed01<f64>>();
            let secs = ((duration.as_secs() as f64) * jitter).ceil() as u64;
            let nanos = ((duration.subsec_nanos() as f64) * jitter).ceil() as u32;
            return Duration::new(secs, nanos);
        })
    }
}
