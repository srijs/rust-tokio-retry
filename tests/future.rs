use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::Future;
use futures::sync::oneshot::spawn;
use tokio::runtime::Runtime;
use tokio_core::reactor::Core;
use tokio_retry::{Error, Retry, RetryIf};

#[test]
fn attempts_just_once() {
    use std::iter::empty;
    let runtime = Runtime::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::spawn(empty(), move || {
        cloned_counter.fetch_add(1, Ordering::SeqCst);
        Err::<(), u64>(42)
    });
    let res = spawn(future, &runtime.executor()).wait();

    assert_eq!(res, Err(Error::OperationError(42)));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn attempts_until_max_retries_exceeded() {
    use tokio_retry::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100).take(2);
    let runtime = Runtime::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::spawn(s, move || {
        cloned_counter.fetch_add(1, Ordering::SeqCst);
        Err::<(), u64>(42)
    });
    let res = spawn(future, &runtime.executor()).wait();

    assert_eq!(res, Err(Error::OperationError(42)));
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[test]
fn attempts_until_success() {
    use tokio_retry::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100);
    let runtime = Runtime::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::spawn(s, move || {
        let previous = cloned_counter.fetch_add(1, Ordering::SeqCst);
        if previous < 3 {
            Err::<(), u64>(42)
        } else {
            Ok::<(), u64>(())
        }
    });
    let res = spawn(future, &runtime.executor()).wait();

    assert_eq!(res, Ok(()));
    assert_eq!(counter.load(Ordering::SeqCst), 4);
}

#[test]
fn compatible_with_tokio_core() {
    use tokio_retry::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100);
    let mut core = Core::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::spawn(s, move || {
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

#[test]
fn attempts_retry_only_if_given_condition_is_true() {
    use tokio_retry::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100).take(5);
    let runtime = Runtime::new().unwrap();
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = RetryIf::spawn(s, move || {
        let previous  = cloned_counter.fetch_add(1, Ordering::SeqCst);
        Err::<(), usize>(previous + 1)
    }, |e: &usize| *e < 3);
    let res = spawn(future, &runtime.executor()).wait();

    assert_eq!(res, Err(Error::OperationError(3)));
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}
