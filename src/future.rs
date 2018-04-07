use std::iter::{Iterator, IntoIterator};
use std::error;
use std::cmp;
use std::fmt;
use std::time::{Duration, Instant};

use futures::{Async, Future, Poll};
use tokio_timer::{Delay, Error as TimerError};
use tokio_timer::timer::Handle;

use super::action::Action;
use super::condition::Condition;

/// Represents the errors possible during the execution of the `RetryFuture`.
#[derive(Debug)]
pub enum Error<E> {
    OperationError(E),
    TimerError(TimerError)
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
    Sleeping(Delay)
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
    Sleeping(Poll<(), TimerError>)
}

/// Future that drives multiple attempts at an action via a retry strategy.
pub struct Retry<I, A> where I: Iterator<Item=Duration>, A: Action {
    retry_if: RetryIf<I, A, fn(&A::Error) -> bool>
}

impl<I, A> Retry<I, A> where I: Iterator<Item=Duration>, A: Action {
    pub fn spawn<T: IntoIterator<IntoIter=I, Item=Duration>>(strategy: T, action: A) -> Retry<I, A> {
        Retry::new(None, strategy, action)
    }

    pub fn spawn_with_handle<T: IntoIterator<IntoIter=I, Item=Duration>>(handle: Handle, strategy: T, action: A) -> Retry<I, A> {
        Retry::new(Some(handle), strategy, action)
    }

    fn new<T: IntoIterator<IntoIter=I, Item=Duration>>(handle: Option<Handle>, strategy: T, action: A) -> Retry<I, A> {
        Retry {
            retry_if: RetryIf::new(handle, strategy, action, (|_| true) as fn(&A::Error) -> bool)
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

/// Future that drives multiple attempts at an action via a retry strategy. Retries are only attempted if
/// the `Error` returned by the future satisfies a given condition.
pub struct RetryIf<I, A, C> where I: Iterator<Item=Duration>, A: Action, C: Condition<A::Error> {
    strategy: I,
    state: RetryState<A>,
    action: A,
    handle: Option<Handle>,
    condition: C
}

impl<I, A, C> RetryIf<I, A, C> where I: Iterator<Item=Duration>, A: Action, C: Condition<A::Error> {
    pub fn spawn<T: IntoIterator<IntoIter=I, Item=Duration>>(
        strategy: T,
        action: A,
        condition: C
    ) -> RetryIf<I, A, C> {
        RetryIf::new(None, strategy, action, condition)
    }

    pub fn spawn_with_handle<T: IntoIterator<IntoIter=I, Item=Duration>>(
        handle: Handle,
        strategy: T,
        action: A,
        condition: C
    ) -> RetryIf<I, A, C> {
        RetryIf::new(Some(handle), strategy, action, condition)
    }

    pub fn new<T: IntoIterator<IntoIter=I, Item=Duration>>(
        handle: Option<Handle>,
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
                let deadline = Instant::now() + duration;
                let handle = self.handle.clone().unwrap_or_else(Handle::current);
                let future = handle.delay(deadline);
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
