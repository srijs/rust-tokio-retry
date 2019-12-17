use std::future::Future;

/// An action can be run multiple times and produces a future.
pub trait Action {
    /// The future that this action produces.
    type Future: Future<Output = Self::Output>;
    /// The item that the future may resolve with.
    type Output;

    fn run(&mut self) -> Self::Future;
}

impl<T: Future, F: FnMut() -> T> Action for F {
    type Future = T;
    type Output = T::Output;

    fn run(&mut self) -> Self::Future {
        self()
    }
}
