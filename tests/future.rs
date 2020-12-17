use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use futures::future::ready;
use futures::Future;
use tokio::runtime::Runtime;
use tokio_retry::{Error, Retry, RetryIf};

fn spawn<O, F: Future<Output = O>>(f: F, runtime: &mut Runtime) -> O {
    runtime.block_on(f)
}

#[test]
fn attempts_just_once() {
    use std::iter::empty;
    let mut runtime = Runtime::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::new(empty(), move || {
        cloned_counter.fetch_add(1, Ordering::SeqCst);
        ready(Err::<(), u64>(42))
    });
    let res = spawn(future, &mut runtime);

    assert_eq!(res, Err(Error::OperationError(42)));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn attempts_until_max_retries_exceeded() {
    use tokio_retry::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100).take(2);
    let mut runtime = Runtime::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::new(s, move || {
        cloned_counter.fetch_add(1, Ordering::SeqCst);
        ready(Err::<(), u64>(42))
    });
    let res = spawn(future, &mut runtime);

    assert_eq!(res, Err(Error::OperationError(42)));
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[test]
fn attempts_until_success() {
    use tokio_retry::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100);
    let mut runtime = Runtime::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::new(s, move || {
        let previous = cloned_counter.fetch_add(1, Ordering::SeqCst);
        ready(if previous < 3 {
            Err::<(), u64>(42)
        } else {
            Ok::<(), u64>(())
        })
    });
    let res = spawn(future, &mut runtime);

    assert_eq!(res, Ok(()));
    assert_eq!(counter.load(Ordering::SeqCst), 4);
}
/*
#[test]
fn compatible_with_tokio_core() {
    use tokio_retry::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100);
    let mut core = Core::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::new(s, move || {
        let previous = cloned_counter.fetch_add(1, Ordering::SeqCst);
        if previous < 3 {
            Err::<(), u64>(42)
        } else {
            Ok::<(), u64>(())
        }
    });
    let res = core.run(future);

    assert_eq!(res, Ok(()));
    assert_eq!(counter.load(Ordering::SeqCst), 4);
}
*/

#[test]
fn attempts_retry_only_if_given_condition_is_true() {
    use tokio_retry::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100).take(5);
    let mut runtime = Runtime::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = RetryIf::new(
        s,
        move || {
            let previous = cloned_counter.fetch_add(1, Ordering::SeqCst);
            ready(Err::<(), usize>(previous + 1))
        },
        |e: &usize| *e < 3,
    );
    let res = spawn(future, &mut runtime);

    assert_eq!(res, Err(Error::OperationError(3)));
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}
