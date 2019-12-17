use std::future::Future;
use std::iter::{IntoIterator, Iterator};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use pin_utils::unsafe_pinned;
use tokio::time::{delay_for, Delay};

use super::action::Action;
use super::condition::Condition;

enum RetryState<F> {
    Running(F),
    Sleeping(Delay),
}
impl<F: Unpin> Unpin for RetryState<F> {}

impl<F> RetryState<F>
where
    F: Future,
{
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> RetryFuturePoll<F> {
        let this = unsafe { self.get_unchecked_mut() };
        match this {
            RetryState::Running(future) => {
                RetryFuturePoll::Running(unsafe { Pin::new_unchecked(future) }.poll(cx))
            }
            RetryState::Sleeping(future) => {
                RetryFuturePoll::Sleeping(unsafe { Pin::new_unchecked(future) }.poll(cx))
            }
        }
    }
}

enum RetryFuturePoll<F>
where
    F: Future,
{
    Running(Poll<F::Output>),
    Sleeping(Poll<()>),
}

/// Future that drives multiple attempts at an action via a retry strategy.
pub struct Retry<I, A>
where
    A: Action,
{
    retry_if: RetryIf<I, A, fn(&A::Output) -> bool>,
}

impl<I, A, U, E> Retry<I, A>
where
    I: Iterator<Item = Duration>,
    A: Action<Output = Result<U, E>>,
    A::Future: Unpin,
{
    unsafe_pinned!(retry_if: RetryIf<I, A, fn(&A::Output) -> bool>);

    pub fn spawn<T: IntoIterator<IntoIter = I, Item = Duration>>(
        strategy: T,
        action: A,
    ) -> Retry<I, A> {
        Retry {
            retry_if: RetryIf::spawn(strategy, action, Result::is_err as fn(&A::Output) -> bool),
        }
    }
}

impl<I, A, U, E> Future for Retry<I, A>
where
    I: Iterator<Item = Duration>,
    A: Action<Output = Result<U, E>>,
    A::Future: Unpin,
{
    type Output = A::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.retry_if().poll(cx)
    }
}

/// Future that drives multiple attempts at an action via a retry strategy. Retries are only attempted if
/// the `Error` returned by the future satisfies a given condition.
pub struct RetryIf<I, A, C>
where
    A: Action,
{
    strategy: I,
    state: RetryState<A::Future>,
    action: A,
    condition: C,
}

impl<I, A: Action, C> Unpin for RetryIf<I, A, C> {}

impl<I, A, C> RetryIf<I, A, C>
where
    I: Iterator<Item = Duration>,
    A: Action,
    C: Condition<A::Output>,
    A::Future: Unpin,
{
    unsafe_pinned!(state: RetryState<A::Future>);

    pub fn spawn<T: IntoIterator<IntoIter = I, Item = Duration>>(
        strategy: T,
        mut action: A,
        condition: C,
    ) -> RetryIf<I, A, C> {
        RetryIf {
            strategy: strategy.into_iter(),
            state: RetryState::Running(action.run()),
            action: action,
            condition: condition,
        }
    }

    fn attempt(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<A::Output> {
        let future = self.action.run();
        self.state = RetryState::Running(future);
        self.poll(cx)
    }

    fn retry(mut self: Pin<&mut Self>, out: A::Output, cx: &mut Context<'_>) -> Poll<A::Output> {
        match self.strategy.next() {
            None => Poll::Ready(out),
            Some(duration) => {
                let future = delay_for(duration);
                self.state = RetryState::Sleeping(future);
                self.poll(cx)
            }
        }
    }
}

impl<I, A, C> Future for RetryIf<I, A, C>
where
    I: Iterator<Item = Duration>,
    A: Action,
    C: Condition<A::Output>,
    A::Future: Unpin,
{
    type Output = A::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.as_mut().state().poll(cx) {
            RetryFuturePoll::Running(poll_result) => match poll_result {
                Poll::Ready(out) => {
                    if self.condition.should_retry(&out) {
                        self.retry(out, cx)
                    } else {
                        Poll::Ready(out)
                    }
                }
                Poll::Pending => Poll::Pending,
            },
            RetryFuturePoll::Sleeping(poll_result) => match poll_result {
                Poll::Ready(_) => self.attempt(cx),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
