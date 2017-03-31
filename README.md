# tokio-retry

Extensible, asynchronous retry behaviours based on [futures](https://crates.io/crates/futures), for the ecosystem of [tokio](https://tokio.rs/) libraries.

[![crates](http://meritbadge.herokuapp.com/tokio-retry)](https://crates.io/crates/tokio-retry)

[Documentation](https://docs.rs/tokio-retry)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tokio-retry = "*"
```

By default, `tokio-retry` will work both with the [`Handle`](https://docs.rs/tokio-core/0.1.4/tokio_core/reactor/struct.Handle.html) type from `tokio-core`, and the [`Timer`](https://docs.rs/tokio-timer/0.1.0/tokio_timer/struct.Timer.html) type from `tokio-timer`. Both of these can be disabled or enabled via cargo feature flags:

```toml
[dependencies.tokio-retry]
version = "*"
default-features = false
# enable only tokio-core compatibility
features = ["tokio_core"]
```

## Examples

```rust
extern crate futures;
extern crate tokio_core;
extern crate tokio_retry;

use tokio_core::reactor::Core;
use tokio_retry::RetryFuture;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

fn action() -> Result<u64, ()> {
    // do some real-world stuff here...
    Ok(42)
}

pub fn main() {
    let mut core = Core::new().unwrap();

    let retry_strategy = ExponentialBackoff::from_millis(10)
        .map(jitter)
        .take(3);
  
    let retry_future = RetryFuture::spawn(core.handle(), retry_strategy, action);
    let retry_result = core.run(retry_future);

    assert_eq!(retry_result, Ok(42));
}
```
