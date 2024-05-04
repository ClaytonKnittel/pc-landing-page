use crate::{
  error::{McError, ThreadSafeError},
  proto::ServerState,
  systemctl::unit::Unit,
};
use std::time::Duration;
use tokio::{
  sync::{Mutex, MutexGuard},
  time::Instant,
};

const REFRESH_RATE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct ServerStatus<U> {
  unit: U,
  last_updated: Instant,
  state: ServerState,
}

impl<U> ServerStatus<U>
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

  pub fn unit_mut(&mut self) -> &mut U {
    &mut self.unit
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

    self.state = match (self.state, self.unit.is_active()) {
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

pub struct ServerController<U> {
  server_status: Mutex<ServerStatus<U>>,
}

impl<U> ServerController<U>
where
  U: Unit + Send + Sync,
{
  pub fn new(unit: U) -> Self {
    Self {
      server_status: ServerStatus::new(unit).into(),
    }
  }

  async fn server_status_guard(
    &self,
  ) -> Result<MutexGuard<'_, ServerStatus<U>>, Box<dyn ThreadSafeError>> {
    let mut status_guard = self.server_status.lock().await;
    status_guard.maybe_update().await?;
    Ok(status_guard)
  }

  pub async fn server_state(&self) -> Result<ServerState, Box<dyn ThreadSafeError>> {
    Ok(self.server_status_guard().await?.state())
  }

  pub async fn boot_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    let boot_fut = {
      let mut guard = self.server_status_guard().await?;
      if guard.state != ServerState::Off {
        return Err(
          McError::InvalidOp(format!("Can't turn server on in {:?} state", guard.state)).into(),
        );
      }
      guard.begin_boot();
      guard.unit_mut().start()
    };

    let exit_status = boot_fut.await?;
    if exit_status.success() {
      Ok(())
    } else {
      let mut guard = self.server_status_guard().await?;
      guard.abort_boot();
      Err(McError::NonzeroExit(exit_status).into())
    }
  }

  pub async fn shutdown_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    let shutdown_fut = {
      let mut guard = self.server_status_guard().await?;
      if guard.state != ServerState::On {
        return Err(
          McError::InvalidOp(format!("Can't turn server off in {:?} state", guard.state)).into(),
        );
      }
      guard.begin_shutdown();
      guard.unit_mut().stop()
    };

    let exit_status = shutdown_fut.await?;
    if exit_status.success() {
      self.server_status_guard().await?.complete_shutdown();
      Ok(())
    } else {
      self.server_status_guard().await?.abort_shutdown();
      Err(McError::NonzeroExit(exit_status).into())
    }
  }
}
