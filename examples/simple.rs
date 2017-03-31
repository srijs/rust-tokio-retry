extern crate futures;
extern crate tokio_core;
extern crate tokio_timer;
extern crate tokio_retry;

use tokio_core::reactor::Core;
use tokio_retry::RetryFuture;
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use std::time::Instant;

pub fn main() {
    let mut core = Core::new().unwrap();

    let retry_strategy = ExponentialBackoff::from_millis(10)
        .map(jitter)
        .take(3);

    let mut last_instant = Instant::now();

    let retry_future = RetryFuture::spawn(core.handle(), retry_strategy, move || {
        let this_instant = Instant::now();

        let duration = this_instant.duration_since(last_instant);
        last_instant = this_instant;

        println!("Actual {:?}", duration);
        Err::<(), ()>(())
    });

    let retry_result = core.run(retry_future);

    assert!(retry_result.is_err());
}