use futures::{Async, Future, Poll};
use std::iter::{Iterator, IntoIterator};
use std::error;
use std::io;
use std::cmp;
use std::fmt;
use std::time::Duration;
use tokio_core::reactor::{Handle, Timeout};

use super::action::Action;
use super::condition::Condition;

/// Represents the errors possible during the execution of the `RetryFuture`.
#[derive(Debug)]
pub enum Error<E> {
    OperationError(E),
    TimerError(io::Error)
}

impl<E: cmp::PartialEq> cmp::PartialEq for Error<E> {
    fn eq(&self, other: &Error<E>) -> bool  {
        match (self, other) {
            (&Error::TimerError(_), _) => false,
            (_, &Error::TimerError(_)) => false,
            (&Error::OperationError(ref left_err), &Error::OperationError(ref right_err)) =>
                left_err.eq(right_err)
        }
    }
}

impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::OperationError(ref err) => err.fmt(formatter),
            Error::TimerError(ref err) => err.fmt(formatter)
        }
    }
}

impl<E: error::Error> error::Error for Error<E> {
    fn description(&self) -> &str {
        match *self {
            Error::OperationError(ref err) => err.description(),
            Error::TimerError(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::OperationError(ref err) => Some(err),
            Error::TimerError(ref err) => Some(err)
        }
    }
}

enum RetryState<A> where A: Action {
    Running(A::Future),
    Sleeping(Timeout)
}

impl<A: Action> RetryState<A> {
    fn poll(&mut self) -> RetryFuturePoll<A> {
        match *self {
            RetryState::Running(ref mut future) =>
                RetryFuturePoll::Running(future.poll()),
            RetryState::Sleeping(ref mut future) =>
                RetryFuturePoll::Sleeping(future.poll())
        }
    }
}

enum RetryFuturePoll<A> where A: Action {
    Running(Poll<A::Item, A::Error>),
    Sleeping(Poll<(), io::Error>)
}

struct RetryAlways;

impl<E> Condition<E> for RetryAlways {
    fn should_retry(&mut self, _error: &E) -> bool {
        true
    }
}

/// Future that drives multiple attempts at an action via a retry strategy.
pub struct Retry<I, A> where I: Iterator<Item=Duration>, A: Action {
    retry_if: RetryIf<I, A, RetryAlways>
}

impl<I, A> Retry<I, A> where I: Iterator<Item=Duration>, A: Action {
    pub fn spawn<T: IntoIterator<IntoIter=I, Item=Duration>>(handle: Handle, strategy: T, action: A) -> Retry<I, A> {
        Retry {
            retry_if: RetryIf::with_condition(handle, strategy, action, RetryAlways)
        }
    }
}

impl<I, A> Future for Retry<I, A> where I: Iterator<Item=Duration>, A: Action {
    type Item = A::Item;
    type Error = Error<A::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.retry_if.poll()
    }
}

pub struct RetryIf<I, A, C> where I: Iterator<Item=Duration>, A: Action, C: Condition<A::Error> {
    strategy: I,
    state: RetryState<A>,
    action: A,
    handle: Handle,
    condition: C
}

impl<I, A, C> RetryIf<I, A, C> where I: Iterator<Item=Duration>, A: Action, C: Condition<A::Error> {
    pub fn with_condition<T: IntoIterator<IntoIter=I, Item=Duration>>(
        handle: Handle,
        strategy: T,
        mut action: A,
        condition: C
    ) -> RetryIf<I, A, C> {
        RetryIf {
            strategy: strategy.into_iter(),
            state: RetryState::Running(action.run()),
            action: action,
            handle: handle,
            condition: condition,
        }
    }

    fn attempt(&mut self) -> Poll<A::Item, Error<A::Error>> {
        let future = self.action.run();
        self.state = RetryState::Running(future);
        self.poll()
    }

    fn retry(&mut self, err: A::Error) -> Poll<A::Item, Error<A::Error>> {
        match self.strategy.next() {
            None => Err(Error::OperationError(err)),
            Some(duration) => {
                let future = Timeout::new(duration, &self.handle)
                    .map_err(Error::TimerError)?;
                self.state = RetryState::Sleeping(future);
                self.poll()
            }
        }
    }
}

impl<I, A, C> Future for RetryIf<I, A, C> where I: Iterator<Item=Duration>, A: Action, C: Condition<A::Error> {
    type Item = A::Item;
    type Error = Error<A::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.state.poll() {
            RetryFuturePoll::Running(poll_result) => match poll_result {
                Ok(async) => Ok(async),
                Err(err) => {
                    if self.condition.should_retry(&err) {
                        self.retry(err)
                    } else {
                        Err(Error::OperationError(err))
                    }
                }
            },
            RetryFuturePoll::Sleeping(poll_result) => match poll_result {
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Ok(Async::Ready(_)) => self.attempt(),
                Err(err) => Err(Error::TimerError(err))
            }
        }
    }
}

#[test]
fn attempts_just_once() {
    use tokio_core::reactor::Core;
    use std::iter::empty;
    let mut core = Core::new().unwrap();
    let mut num_calls = 0;
    let res = {
        let fut = Retry::spawn(core.handle(), empty(), || {
            num_calls += 1;
            Err::<(), u64>(42)
        });
        core.run(fut)
    };

    assert_eq!(res, Err(Error::OperationError(42)));
    assert_eq!(num_calls, 1);
}

#[test]
fn attempts_until_max_retries_exceeded() {
    use tokio_core::reactor::Core;
    use super::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100).take(2);
    let mut core = Core::new().unwrap();
    let mut num_calls = 0;
    let res = {
        let fut = Retry::spawn(core.handle(), s, || {
            num_calls += 1;
            Err::<(), u64>(42)
        });
        core.run(fut)
    };

    assert_eq!(res, Err(Error::OperationError(42)));
    assert_eq!(num_calls, 3);
}

#[test]
fn attempts_until_success() {
    use tokio_core::reactor::Core;
    use super::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100);
    let mut core = Core::new().unwrap();
    let mut num_calls = 0;
    let res = {
        let fut = Retry::spawn(core.handle(), s, || {
            num_calls += 1;
            if num_calls < 4 {
                Err::<(), u64>(42)
            } else {
                Ok::<(), u64>(())
            }
        });
        core.run(fut)
    };

    assert_eq!(res, Ok(()));
    assert_eq!(num_calls, 4);
}

#[test]
fn attempts_retry_only_if_given_condition_is_true() {
    use tokio_core::reactor::Core;
    use super::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100).take(5);
    let mut core = Core::new().unwrap();
    let mut num_calls = 0;
    let res = {
        let action = || {
            num_calls += 1;
            Err::<(), u64>(num_calls)
        };
        let fut = RetryIf::with_condition(core.handle(), s, action, |e: &u64| *e < 3);
        core.run(fut)
    };

    assert_eq!(res, Err(Error::OperationError(3)));
    assert_eq!(num_calls, 3);
}