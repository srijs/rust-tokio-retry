use either::Either;
use futures::{Async, IntoFuture, Future, Poll};
use std::error::Error;
use std::cmp;
use std::fmt;
use tokio_timer::{Sleep, Timer, TimerError};

use super::strategy::RetryStrategy;

/// Represents the errors possible during the execution of the `RetryFuture`.
#[derive(Debug)]
pub enum RetryError<E> {
    OperationError(E),
    TimerError(TimerError)
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

enum RetryState<A> where A: IntoFuture {
    Running(A::Future),
    Sleeping(Sleep)
}

/// Future that drives multiple attempts at an action via a retry strategy.
pub struct RetryFuture<S, A, F> where S: RetryStrategy, A: IntoFuture, F: FnMut() -> A {
    timer: Timer,
    strategy: S,
    state: RetryState<A>,
    action: F
}

pub fn retry<S, A, F>(strategy: S, timer: Timer, action: F) -> RetryFuture<S, A, F> where S: RetryStrategy, A: IntoFuture, F: FnMut() -> A {
    RetryFuture::spawn(strategy, timer, action)
}

impl<S, A, F> RetryFuture<S, A, F> where S: RetryStrategy, A: IntoFuture, F: FnMut() -> A {
    fn spawn(strategy: S, timer: Timer, mut action: F) -> RetryFuture<S, A, F> {
        RetryFuture {
            timer: timer,
            strategy: strategy,
            state: RetryState::Running(action().into_future()),
            action: action
        }
    }

    fn attempt(&mut self) -> Poll<A::Item, RetryError<A::Error>> {
        let future = (self.action)().into_future();
        self.state = RetryState::Running(future);
        return self.poll();
    }

    fn retry(&mut self, err: A::Error) -> Poll<A::Item, RetryError<A::Error>> {
        match self.strategy.delay() {
            None => Err(RetryError::OperationError(err)),
            Some(duration) => {
                let future = self.timer.sleep(duration);
                self.state = RetryState::Sleeping(future);
                return self.poll();
            }
        }
    }
}

impl<S, A, F> Future for RetryFuture<S, A, F> where S: RetryStrategy, A: IntoFuture, F: FnMut() -> A {
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
                Async::Ready(()) => self.attempt()
            }
        }
    }
}

#[test]
fn attempts_just_once() {
    use std::default::Default;
    use super::strategies::NoRetry;
    let s = NoRetry{};
    let mut num_calls = 0;
    let res = s.run(Timer::default(), || {
        num_calls += 1;
        Err::<(), u64>(42)
    }).wait();

    assert_eq!(res, Err(RetryError::OperationError(42)));
    assert_eq!(num_calls, 1);
}

#[test]
fn attempts_until_max_retries_exceeded() {
    use std::default::Default;
    use std::time::Duration;
    use super::strategies::FixedInterval;
    let s = FixedInterval::new(Duration::from_millis(100)).limit_retries(2);
    let mut num_calls = 0;
    let res = s.run(Timer::default(), || {
        num_calls += 1;
        Err::<(), u64>(42)
    }).wait();

    assert_eq!(res, Err(RetryError::OperationError(42)));
    assert_eq!(num_calls, 3);
}

#[test]
fn attempts_until_success() {
    use std::default::Default;
    use std::time::Duration;
    use super::strategies::FixedInterval;
    let s = FixedInterval::new(Duration::from_millis(100));
    let mut num_calls = 0;
    let res = s.run(Timer::default(), || {
        num_calls += 1;
        if num_calls < 4 {
            Err::<(), u64>(42)
        } else {
            Ok::<(), u64>(())
        }
    }).wait();

    assert_eq!(res, Ok(()));
    assert_eq!(num_calls, 4);
}
