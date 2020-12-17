use futures::future::{IntoFuture, TryFuture, TryFutureExt};
use futures::Future;
use std::result::Result;

/// An action can be run multiple times and produces a future.
pub trait Action {
    /// The future that this action produces.
    type Future: Future<Output = Result<Self::Item, Self::Error>>
        + TryFuture<Ok = Self::Item, Error = Self::Error>;
    /// The item that the future may resolve with.
    type Item;
    type Error;

    fn run(&mut self) -> Self::Future;
}

impl<O, E, T: TryFuture<Ok = O, Error = E>, F: FnMut() -> T> Action for F {
    type Future = IntoFuture<T>;
    type Item = T::Ok;
    type Error = T::Error;

    fn run(&mut self) -> Self::Future {
        self().into_future()
    }
}
