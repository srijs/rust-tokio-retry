use std::time::Duration;
use rand::{random, Closed01};

mod fixed_interval;
mod exponential_backoff;

pub use self::fixed_interval::FixedInterval;
pub use self::exponential_backoff::ExponentialBackoff;

pub fn jitter(duration: Duration) -> Duration {
    let Closed01(jitter) = random::<Closed01<f64>>();
    let secs = ((duration.as_secs() as f64) * jitter).ceil() as u64;
    let nanos = ((duration.subsec_nanos() as f64) * jitter).ceil() as u32;
    return Duration::new(secs, nanos);
}
