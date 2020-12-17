/// Specifies under which conditions a retry is attempted.
pub trait Condition<E>: Unpin {
    fn should_retry(&mut self, error: &E) -> bool;
}

impl<E, F: FnMut(&E) -> bool + Unpin> Condition<E> for F {
    fn should_retry(&mut self, error: &E) -> bool {
        self(error)
    }
}
