use std::{
  io::{Error, ErrorKind},
  process::ExitStatus,
};

use tokio::{
  io::AsyncReadExt,
  process::{Child, Command},
};

use super::util::SYSTEMCTL_PATH;

/// Invokes `systemctl $args`
fn spawn_child(args: Vec<&str>) -> std::io::Result<Child> {
  Command::new(std::env::var("SYSTEMCTL_PATH").unwrap_or(SYSTEMCTL_PATH.into()))
    .args(args)
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::null())
    .spawn()
}

/// Invokes `systemctl $args` silently
pub async fn systemctl(args: Vec<&str>) -> std::io::Result<ExitStatus> {
  spawn_child(args)?.wait().await
}

/// Invokes `systemctl $args` and captures stdout stream
pub async fn systemctl_capture(args: Vec<&str>) -> std::io::Result<String> {
  let mut child = spawn_child(args)?;
  match child.wait().await?.code() {
    Some(0) => {} // success
    Some(1) => {} // success -> Ok(Unit not found)
    Some(3) => {} // success -> Ok(unit is inactive and/or dead)
    Some(4) => {
      return Err(Error::new(
        ErrorKind::PermissionDenied,
        "Missing Privileges or Unit not found",
      ))
    }
    // unknown errorcodes
    Some(code) => {
      return Err(Error::new(
        // TODO: Maybe a better ErrorKind, none really seem to fit
        ErrorKind::Other,
        format!("Process exited with code: {code}"),
      ));
    }
    None => {
      return Err(Error::new(
        ErrorKind::Interrupted,
        "Process terminated by signal",
      ))
    }
  }

  let mut stdout: Vec<u8> = Vec::new();
  let size = child.stdout.unwrap().read_to_end(&mut stdout).await?;

  if size > 0 {
    if let Ok(s) = String::from_utf8(stdout) {
      return Ok(s);
    } else {
      return Err(Error::new(
        ErrorKind::InvalidData,
        "Invalid utf8 data in stdout",
      ));
    }
  }

  // if this is reached all if's above did not work
  Err(Error::new(
    ErrorKind::UnexpectedEof,
    "systemctl stdout empty",
  ))
}

/// Forces given `unit` to (re)start
pub async fn restart(unit: &str) -> std::io::Result<ExitStatus> {
  systemctl(vec!["restart", unit]).await
}

/// Forces given `unit` to start
pub async fn start(unit: &str) -> std::io::Result<ExitStatus> {
  systemctl(vec!["start", unit]).await
}

/// Forces given `unit` to stop
pub async fn stop(unit: &str) -> std::io::Result<ExitStatus> {
  systemctl(vec!["stop", unit]).await
}

/// Triggers reload for given `unit`
pub async fn reload(unit: &str) -> std::io::Result<ExitStatus> {
  systemctl(vec!["reload", unit]).await
}

/// Triggers reload or restarts given `unit`
pub async fn reload_or_restart(unit: &str) -> std::io::Result<ExitStatus> {
  systemctl(vec!["reload-or-restart", unit]).await
}

/// Enable given `unit` to start at boot
pub async fn enable(unit: &str) -> std::io::Result<ExitStatus> {
  systemctl(vec!["enable", unit]).await
}

/// Disable given `unit` to start at boot
pub async fn disable(unit: &str) -> std::io::Result<ExitStatus> {
  systemctl(vec!["disable", unit]).await
}

/// Returns raw status from `systemctl status $unit` call
pub async fn status(unit: &str) -> std::io::Result<String> {
  systemctl_capture(vec!["status", unit]).await
}

/// Invokes systemctl `cat` on given `unit`
pub async fn cat(unit: &str) -> std::io::Result<String> {
  systemctl_capture(vec!["cat", unit]).await
}

/// Returns `true` if given `unit` is actively running
pub async fn is_active(unit: &str) -> std::io::Result<bool> {
  let status = systemctl_capture(vec!["is-active", unit]).await?;
  Ok(status.trim_end().eq("active"))
}

/// Isolates given unit, only self and its dependencies are
/// now actively running
pub async fn isolate(unit: &str) -> std::io::Result<ExitStatus> {
  systemctl(vec!["isolate", unit]).await
}

/// Freezes (halts) given unit.
/// This operation might not be feasible.
pub async fn freeze(unit: &str) -> std::io::Result<ExitStatus> {
  systemctl(vec!["freeze", unit]).await
}

/// Unfreezes given unit (recover from halted state).
/// This operation might not be feasible.
pub async fn unfreeze(unit: &str) -> std::io::Result<ExitStatus> {
  systemctl(vec!["thaw", unit]).await
}
