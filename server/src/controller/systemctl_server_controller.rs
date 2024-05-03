use crate::{
  error::{McError, ThreadSafeError},
  proto::ServerState,
  systemctl::{self, Unit},
};
use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, MutexGuard};

use super::controller_interface::ServerController;

const MC_SERVER_SERVICE: &str = "mc_server.service";
const REFRESH_RATE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct SystemctlServerStatus {
  unit: Option<Unit>,
  last_updated: Instant,
  state: ServerState,
}

impl SystemctlServerStatus {
  fn new() -> Self {
    Self {
      unit: None,
      last_updated: Instant::now(),
      state: ServerState::Unknown,
    }
  }

  pub fn state(&mut self) -> ServerState {
    self.state
  }

  fn begin_boot(&mut self) {
    debug_assert_eq!(self.state, ServerState::Off);
    self.state = ServerState::Booting;
  }

  fn abort_boot(&mut self) {
    debug_assert_eq!(self.state, ServerState::Booting);
    self.state = ServerState::Off;
  }

  fn begin_shutdown(&mut self) {
    debug_assert_eq!(self.state, ServerState::On);
    self.state = ServerState::Shutdown;
  }

  fn complete_shutdown(&mut self) {
    debug_assert_eq!(self.state, ServerState::Shutdown);
    self.state = ServerState::Off;
  }

  fn abort_shutdown(&mut self) {
    debug_assert_eq!(self.state, ServerState::Shutdown);
    self.state = ServerState::On;
  }

  async fn do_update(&mut self, now: Instant) -> Result<(), Box<dyn ThreadSafeError>> {
    self.unit = Some(Self::refresh_unit().await?);
    self.last_updated = now;

    self.state = match (self.state, self.unit.as_ref().unwrap().active) {
      (ServerState::Shutdown, _) => ServerState::Shutdown,
      (ServerState::Booting, false) => ServerState::Booting,
      (_, false) => ServerState::Off,
      (_, true) => ServerState::On,
    };
    Ok(())
  }

  async fn maybe_update(&mut self) -> Result<(), Box<dyn ThreadSafeError>> {
    let now = Instant::now();
    if self.unit.is_some() && now < self.last_updated + REFRESH_RATE {
      return Ok(());
    }
    self.do_update(now).await
  }

  async fn refresh_unit() -> Result<Unit, Box<dyn ThreadSafeError>> {
    Unit::from_systemctl(MC_SERVER_SERVICE)
      .await
      .map_err(Box::from)
  }
}

pub struct SystemctlServerController {
  server_status: Mutex<SystemctlServerStatus>,
}

impl SystemctlServerController {
  pub fn new() -> Self {
    Self {
      server_status: SystemctlServerStatus::new().into(),
    }
  }

  async fn server_status_guard(
    &self,
  ) -> Result<MutexGuard<'_, SystemctlServerStatus>, Box<dyn ThreadSafeError>> {
    let mut status_guard = self.server_status.lock().await;
    status_guard.maybe_update().await?;
    Ok(status_guard)
  }
}

#[async_trait]
impl ServerController for SystemctlServerController {
  async fn server_state(&self) -> Result<ServerState, Box<dyn ThreadSafeError>> {
    Ok(self.server_status_guard().await?.state())
  }

  async fn boot_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    {
      let mut guard = self.server_status_guard().await?;
      if guard.state != ServerState::Off {
        return Err(
          McError::InvalidOp(format!("Can't turn server on in {:?} state", guard.state)).into(),
        );
      }
      guard.begin_boot();
    }

    let exit_status = systemctl::start(MC_SERVER_SERVICE).await?;
    if exit_status.success() {
      Ok(())
    } else {
      let mut guard = self.server_status_guard().await?;
      guard.abort_boot();
      Err(McError::NonzeroExit(exit_status).into())
    }
  }

  async fn shutdown_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    {
      let mut guard = self.server_status_guard().await?;
      if guard.state != ServerState::On {
        return Err(
          McError::InvalidOp(format!("Can't turn server off in {:?} state", guard.state)).into(),
        );
      }
      guard.begin_shutdown();
    }

    let exit_status = systemctl::stop(MC_SERVER_SERVICE).await?;
    if exit_status.success() {
      self.server_status_guard().await?.complete_shutdown();
      Ok(())
    } else {
      self.server_status_guard().await?.abort_shutdown();
      Err(McError::NonzeroExit(exit_status).into())
    }
  }
}
