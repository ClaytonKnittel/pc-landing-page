use crate::{
  error::{McError, ThreadSafeError},
  proto::ServerState,
  systemctl::{self, unit::Unit},
};
use async_trait::async_trait;
use std::time::Duration;
use tokio::{
  sync::{Mutex, MutexGuard},
  time::Instant,
};

use super::controller_interface::ServerController;

const REFRESH_RATE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct SystemctlServerStatus<U> {
  unit: U,
  last_updated: Instant,
  state: ServerState,
}

impl<U> SystemctlServerStatus<U>
where
  U: Unit,
{
  fn new(unit: U) -> Self {
    Self {
      unit,
      last_updated: Instant::now(),
      state: ServerState::Unknown,
    }
  }

  pub fn unit(&self) -> &U {
    &self.unit
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
    self.unit.refresh().await?;
    self.last_updated = now;

    self.state = match (self.state, self.unit.is_active().await) {
      (ServerState::Shutdown, _) => ServerState::Shutdown,
      (ServerState::Booting, false) => ServerState::Booting,
      (_, false) => ServerState::Off,
      (_, true) => ServerState::On,
    };
    Ok(())
  }

  async fn maybe_update(&mut self) -> Result<(), Box<dyn ThreadSafeError>> {
    let now = Instant::now();
    if now < self.last_updated + REFRESH_RATE {
      return Ok(());
    }
    self.do_update(now).await
  }
}

pub struct SystemctlServerController<U> {
  server_status: Mutex<SystemctlServerStatus<U>>,
}

impl<U> SystemctlServerController<U>
where
  U: Unit,
{
  pub fn new(unit: U) -> Self {
    Self {
      server_status: SystemctlServerStatus::new(unit).into(),
    }
  }

  async fn server_status_guard(
    &self,
  ) -> Result<MutexGuard<'_, SystemctlServerStatus<U>>, Box<dyn ThreadSafeError>> {
    let mut status_guard = self.server_status.lock().await;
    status_guard.maybe_update().await?;
    Ok(status_guard)
  }
}

#[async_trait]
impl<U> ServerController for SystemctlServerController<U>
where
  U: Unit + Send + Sync,
{
  async fn server_state(&self) -> Result<ServerState, Box<dyn ThreadSafeError>> {
    Ok(self.server_status_guard().await?.state())
  }

  async fn boot_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    let name = {
      let mut guard = self.server_status_guard().await?;
      if guard.state != ServerState::Off {
        return Err(
          McError::InvalidOp(format!("Can't turn server on in {:?} state", guard.state)).into(),
        );
      }
      guard.begin_boot();
      guard.unit.name().to_owned()
    };

    // TODO: remove this dependency
    let exit_status = systemctl::commands::start(&name).await?;
    if exit_status.success() {
      Ok(())
    } else {
      let mut guard = self.server_status_guard().await?;
      guard.abort_boot();
      Err(McError::NonzeroExit(exit_status).into())
    }
  }

  async fn shutdown_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    let name = {
      let mut guard = self.server_status_guard().await?;
      if guard.state != ServerState::On {
        return Err(
          McError::InvalidOp(format!("Can't turn server off in {:?} state", guard.state)).into(),
        );
      }
      guard.begin_shutdown();
      guard.unit.name().to_owned()
    };

    // TODO: remove this dependency
    let exit_status = systemctl::commands::stop(&name).await?;
    if exit_status.success() {
      self.server_status_guard().await?.complete_shutdown();
      Ok(())
    } else {
      self.server_status_guard().await?.abort_shutdown();
      Err(McError::NonzeroExit(exit_status).into())
    }
  }
}
