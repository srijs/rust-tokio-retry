use futures::{Async, Future, Poll};
use std::iter::{Iterator, IntoIterator};
use std::error::Error;
use std::io;
use std::cmp;
use std::fmt;
use std::time::Duration;
use tokio_core::reactor::{Handle, Timeout};

use super::Action;

/// Represents the errors possible during the execution of the `RetryFuture`.
#[derive(Debug)]
pub enum RetryError<E> {
    OperationError(E),
    TimerError(io::Error)
}

impl<E: cmp::PartialEq> cmp::PartialEq for RetryError<E> {
    fn eq(&self, other: &RetryError<E>) -> bool  {
        match (self, other) {
            (&RetryError::TimerError(_), _) => false,
            (_, &RetryError::TimerError(_)) => false,
            (&RetryError::OperationError(ref left_err), &RetryError::OperationError(ref right_err)) =>
                left_err.eq(right_err)
        }
    }
}

impl<E: fmt::Display> fmt::Display for RetryError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            RetryError::OperationError(ref err) => err.fmt(formatter),
            RetryError::TimerError(ref err) => err.fmt(formatter)
        }
    }
}

impl<E: Error> Error for RetryError<E> {
    fn description(&self) -> &str {
        match *self {
            RetryError::OperationError(ref err) => err.description(),
            RetryError::TimerError(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            RetryError::OperationError(ref err) => Some(err),
            RetryError::TimerError(ref err) => Some(err)
        }
    }
}

enum RetryState<A> where A: Action {
    Running(A::Future),
    Sleeping(Timeout)
}

/// Future that drives multiple attempts at an action via a retry strategy.
pub struct RetryFuture<I, A> where I: Iterator<Item=Duration>, A: Action {
    strategy: I,
    state: RetryState<A>,
    action: A,
    handle: Handle
}

impl<I, A> RetryFuture<I, A> where I: Iterator<Item=Duration>, A: Action {
    pub fn spawn<T: IntoIterator<IntoIter=I, Item=Duration>>(handle: Handle, strategy: T, mut action: A) -> RetryFuture<I, A> {
        RetryFuture {
            strategy: strategy.into_iter(),
            state: RetryState::Running(action.run()),
            action: action,
            handle: handle
        }
    }

    fn attempt(&mut self) -> Poll<A::Item, RetryError<A::Error>> {
        let future = self.action.run();
        self.state = RetryState::Running(future);
        return self.poll();
    }

    fn retry(&mut self, err: A::Error) -> Poll<A::Item, RetryError<A::Error>> {
        match self.strategy.next() {
            None => Err(RetryError::OperationError(err)),
            Some(duration) => {
                let future = Timeout::new(duration, &self.handle)
                    .map_err(RetryError::TimerError)?;
                self.state = RetryState::Sleeping(future);
                return self.poll();
            }
        }
    }
}

enum Either<A, B> {
    Left(A),
    Right(B)
}

impl<I, A> Future for RetryFuture<I, A> where I: Iterator<Item=Duration>, A: Action {
    type Item = A::Item;
    type Error = RetryError<A::Error>;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let result = match self.state {
            RetryState::Running(ref mut future) =>
                Either::Left(future.poll()),
            RetryState::Sleeping(ref mut future) =>
                Either::Right(future.poll().map_err(RetryError::TimerError))
        };

        match result {
            Either::Left(poll_result) => match poll_result {
                Ok(async) => Ok(async),
                Err(err) => self.retry(err)
            },
            Either::Right(poll_result) => match poll_result? {
                Async::NotReady => Ok(Async::NotReady),
                Async::Ready(_) => self.attempt()
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
        let fut = RetryFuture::spawn(core.handle(), empty(), || {
            num_calls += 1;
            Err::<(), u64>(42)
        });
        core.run(fut)
    };

    assert_eq!(res, Err(RetryError::OperationError(42)));
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
        let fut = RetryFuture::spawn(core.handle(), s, || {
            num_calls += 1;
            Err::<(), u64>(42)
        });
        core.run(fut)
    };

    assert_eq!(res, Err(RetryError::OperationError(42)));
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
        let fut = RetryFuture::spawn(core.handle(), s, || {
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
