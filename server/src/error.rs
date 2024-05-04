use std::{error, fmt::Display, process::ExitStatus};

pub type McResult<T> = Result<T, McError>;

#[derive(Debug)]
pub enum McError {
  NonzeroExit(ExitStatus),
  InvalidOp(String),
}

impl Display for McError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      McError::NonzeroExit(exit_status) => {
        write!(f, "Nonzero exit: {exit_status}")
      }
      McError::InvalidOp(msg) => {
        write!(f, "Invalid operation: {msg}")
      }
    }
  }
}

impl error::Error for McError {}

pub trait ThreadSafeError: error::Error + Send + Sync {}

impl<T> ThreadSafeError for T where T: error::Error + Send + Sync {}

impl<T> From<T> for Box<dyn ThreadSafeError>
where
  T: ThreadSafeError + Sized + 'static,
{
  fn from(value: T) -> Self {
    Box::new(value)
  }
}
