use std::{error, fmt::Display, process::ExitStatus};

#[derive(Debug)]
pub enum McError {
  NonzeroExit(ExitStatus),
}

impl Display for McError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      McError::NonzeroExit(exit_status) => {
        write!(f, "Nonzero exit: {exit_status}")
      }
    }
  }
}

impl error::Error for McError {}
