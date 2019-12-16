use std::iter::{Iterator, IntoIterator};
use std::time::Duration;

use futures::{Async, Future, Poll};
use futures03::future::{FutureExt, TryFutureExt};

use super::action::Action;
use super::condition::Condition;

enum RetryState<A> where A: Action {
    Running(A::Future),
    Sleeping(tokio::time::Delay)
}

impl<A: Action> RetryState<A> {
    fn poll(&mut self) -> RetryFuturePoll<A> {
        match *self {
            RetryState::Running(ref mut future) =>
                RetryFuturePoll::Running(future.poll()),
            RetryState::Sleeping(ref mut future) =>
                RetryFuturePoll::Sleeping(future.unit_error().compat().poll())
        }
    }
}

enum RetryFuturePoll<A> where A: Action {
    Running(Poll<A::Item, A::Error>),
    Sleeping(Poll<(), ()>)
}

/// Future that drives multiple attempts at an action via a retry strategy.
pub struct Retry<I, A> where I: Iterator<Item=Duration>, A: Action {
    retry_if: RetryIf<I, A, fn(&A::Error) -> bool>
}

impl<I, A> Retry<I, A> where I: Iterator<Item=Duration>, A: Action {
    pub fn spawn<T: IntoIterator<IntoIter=I, Item=Duration>>(strategy: T, action: A) -> Retry<I, A> {
        Retry {
            retry_if: RetryIf::spawn(strategy, action, (|_| true) as fn(&A::Error) -> bool)
        }
    }
}

impl<I, A> Future for Retry<I, A> where I: Iterator<Item=Duration>, A: Action {
    type Item = A::Item;
    type Error = A::Error;

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
    condition: C
}

impl<I, A, C> RetryIf<I, A, C> where I: Iterator<Item=Duration>, A: Action, C: Condition<A::Error> {
    pub fn spawn<T: IntoIterator<IntoIter=I, Item=Duration>>(
        strategy: T,
        mut action: A,
        condition: C
    ) -> RetryIf<I, A, C> {
        RetryIf {
            strategy: strategy.into_iter(),
            state: RetryState::Running(action.run()),
            action: action,
            condition: condition,
        }
    }

    fn attempt(&mut self) -> Poll<A::Item, A::Error> {
        let future = self.action.run();
        self.state = RetryState::Running(future);
        self.poll()
    }

    fn retry(&mut self, err: A::Error) -> Poll<A::Item, A::Error> {
        match self.strategy.next() {
            None => Err(err),
            Some(duration) => {
                let future = tokio::time::delay_for(duration);
                self.state = RetryState::Sleeping(future);
                self.poll()
            }
        }
    }
}

impl<I, A, C> Future for RetryIf<I, A, C> where I: Iterator<Item=Duration>, A: Action, C: Condition<A::Error> {
    type Item = A::Item;
    type Error = A::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.state.poll() {
            RetryFuturePoll::Running(poll_result) => match poll_result {
                Ok(ok) => Ok(ok),
                Err(err) => {
                    if self.condition.should_retry(&err) {
                        self.retry(err)
                    } else {
                        Err(err)
                    }
                }
            },
            RetryFuturePoll::Sleeping(poll_result) => match poll_result {
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Ok(Async::Ready(_)) => self.attempt(),
                Err(()) => unreachable!(), // `Delay` never errors.
            }
        }
    }
}
