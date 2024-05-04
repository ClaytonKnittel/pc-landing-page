#![allow(dead_code)]

//! Crate to manage and monitor services through `systemctl`   
//! Homepage: <https://github.com/gwbres/systemctl>
use async_trait::async_trait;
use std::{io::ErrorKind, pin::Pin, process::ExitStatus};
use strum_macros::EnumString;

use crate::error::ThreadSafeError;

use super::util::ThreadSafeFuture;

/// `AutoStartStatus` describes the Unit current state
#[derive(Copy, Clone, PartialEq, Eq, EnumString, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AutoStartStatus {
  #[strum(serialize = "static")]
  Static,
  #[strum(serialize = "enabled")]
  Enabled,
  #[strum(serialize = "enabled-runtime")]
  EnabledRuntime,
  #[strum(serialize = "disabled")]
  #[default]
  Disabled,
  #[strum(serialize = "generated")]
  Generated,
  #[strum(serialize = "indirect")]
  Indirect,
  #[strum(serialize = "transient")]
  Transient,
}

/// `Type` describes a Unit declaration Type in systemd
#[derive(Copy, Clone, PartialEq, Eq, EnumString, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Type {
  #[strum(serialize = "automount")]
  AutoMount,
  #[strum(serialize = "mount")]
  Mount,
  #[strum(serialize = "service")]
  #[default]
  Service,
  #[strum(serialize = "scope")]
  Scope,
  #[strum(serialize = "socket")]
  Socket,
  #[strum(serialize = "slice")]
  Slice,
  #[strum(serialize = "timer")]
  Timer,
  #[strum(serialize = "path")]
  Path,
  #[strum(serialize = "target")]
  Target,
}

/// `State` describes a Unit current state
#[derive(Copy, Clone, PartialEq, Eq, EnumString, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum State {
  #[strum(serialize = "masked")]
  #[default]
  Masked,
  #[strum(serialize = "loaded")]
  Loaded,
}

/// Doc describes types of documentation possibly
/// available for a systemd `unit`
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Doc {
  /// Man page is available
  Man(String),
  /// Webpage URL is indicated
  Url(String),
}

impl Doc {
  /// Unwrapps self as `Man` page
  pub fn as_man(&self) -> Option<&str> {
    match self {
      Doc::Man(s) => Some(s),
      _ => None,
    }
  }
  /// Unwrapps self as webpage `Url`
  pub fn as_url(&self) -> Option<&str> {
    match self {
      Doc::Url(s) => Some(s),
      _ => None,
    }
  }
}

impl std::str::FromStr for Doc {
  type Err = std::io::Error;
  /// Builds `Doc` from systemd status descriptor
  fn from_str(status: &str) -> Result<Self, Self::Err> {
    let items: Vec<&str> = status.split(':').collect();
    if items.len() != 2 {
      return Err(std::io::Error::new(
        ErrorKind::InvalidData,
        "malformed doc descriptor",
      ));
    }
    match items[0] {
      "man" => {
        let content: Vec<&str> = items[1].split('(').collect();
        Ok(Doc::Man(content[0].to_string()))
      }
      "http" => Ok(Doc::Url("http:".to_owned() + items[1].trim())),
      "https" => Ok(Doc::Url("https:".to_owned() + items[1].trim())),
      _ => Err(std::io::Error::new(
        ErrorKind::InvalidData,
        "unknown type of doc",
      )),
    }
  }
}

pub type AsyncResult<T> =
  Pin<Box<dyn ThreadSafeFuture<Output = Result<T, Box<dyn ThreadSafeError>>>>>;

#[async_trait]
pub trait Unit {
  fn name(&self) -> &str;

  /// Updates the `Unit` by rereading its state.
  async fn refresh(&mut self) -> Result<(), Box<dyn ThreadSafeError>>;

  fn restart(&mut self) -> AsyncResult<ExitStatus>;

  fn start(&mut self) -> AsyncResult<ExitStatus>;

  fn stop(&mut self) -> AsyncResult<ExitStatus>;

  fn reload(&mut self) -> AsyncResult<ExitStatus>;

  fn reload_or_restart(&mut self) -> AsyncResult<ExitStatus>;

  /// Enable Self to start at boot
  fn enable(&mut self) -> AsyncResult<ExitStatus>;

  /// Disable Self to start at boot
  fn disable(&mut self) -> AsyncResult<ExitStatus>;

  /// Returns verbose status for Self
  fn status(&self) -> AsyncResult<String>;

  /// Returns `true` if Self is actively running
  fn is_active(&self) -> bool;

  /// `Isolate` Self, meaning stops all other units but self and its
  /// dependencies
  fn isolate(&mut self) -> AsyncResult<ExitStatus>;

  /// `Freezes` Self, halts self and CPU load will no longer be dedicated to
  /// its execution.  This operation might not be feasible.  `unfreeze()` is
  /// the mirror operation
  fn freeze(&mut self) -> AsyncResult<ExitStatus>;

  /// `Unfreezes` Self, exists halted state.  This operation might not be
  /// feasible.
  fn unfreeze(&mut self) -> AsyncResult<ExitStatus>;

  /// Returns `true` if given `unit` exists, ie., service could be or is
  /// actively deployed and manageable by systemd
  fn exists(&self) -> AsyncResult<bool>;
}
