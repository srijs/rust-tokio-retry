use std::cmp;
use std::error;
use std::fmt;
use tokio::time::Error as TimerError;

/// Represents the errors possible during the execution of the `RetryFuture`.
#[derive(Debug)]
pub enum Error<E> {
    OperationError(E),
    TimerError(TimerError),
}

impl<E: cmp::PartialEq> cmp::PartialEq for Error<E> {
    fn eq(&self, other: &Error<E>) -> bool {
        match (self, other) {
            (&Error::TimerError(_), _) => false,
            (_, &Error::TimerError(_)) => false,
            (&Error::OperationError(ref left_err), &Error::OperationError(ref right_err)) => {
                left_err.eq(right_err)
            }
        }
    }
}

impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::OperationError(ref err) => err.fmt(formatter),
            Error::TimerError(ref err) => err.fmt(formatter),
        }
    }
}

impl<E: error::Error> error::Error for Error<E> {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::OperationError(ref err) => Some(err),
            Error::TimerError(ref err) => Some(err),
        }
    }
}
