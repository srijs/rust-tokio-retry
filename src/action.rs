use futures::{IntoFuture, Future};

pub trait Action {
    type Item;
    type Error;
    type Future: Future<Item=Self::Item, Error=Self::Error>;

    fn run(&mut self) -> Self::Future;
}

pub struct ActionFn<F> {
    action: F
}

impl<A: IntoFuture, F: FnMut() -> A> ActionFn<F> {
    pub fn new(action: F) -> ActionFn<F> {
        ActionFn{action: action}
    }
}

impl<A: IntoFuture, F: FnMut() -> A> Action for ActionFn<F> {
    type Item = <A::Future as Future>::Item;
    type Error = <A::Future as Future>::Error;
    type Future = A::Future;

    fn run(&mut self) -> Self::Future {
        (self.action)().into_future()
    }
}
