/// Specifies under which conditions a retry is attempted.
pub trait Condition<T> {
    fn should_retry(&mut self, out: &T) -> bool;
}

impl<T, F: FnMut(&T) -> bool> Condition<T> for F {
    fn should_retry(&mut self, out: &T) -> bool {
        self(out)
    }
}
