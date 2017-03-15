use futures::{IntoFuture, Future};

pub trait Action {
    type Item;
    type Error;
    type Future: Future<Item=Self::Item, Error=Self::Error>;

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
