use futures::{IntoFuture, Future};

/// An action can be run multiple times and produces a future.
pub trait Action {
    /// The future that this action produces.
    type Future: Future<Item=Self::Item, Error=Self::Error>;
    /// The item that the future may resolve with.
    type Item;
    /// The error that the future may resolve with.
    type Error;

    fn run(&mut self) -> Self::Future;
}

impl<T: IntoFuture, F: FnMut() -> T> Action for F {
    type Item = T::Item;
    type Error = T::Error;
    type Future = T::Future;

    fn run(&mut self) -> Self::Future {
        self().into_future()
    }
}
