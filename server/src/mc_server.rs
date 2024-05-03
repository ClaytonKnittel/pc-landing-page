use crate::{
  error::{McError, ThreadSafeError},
  proto::ServerState,
  systemctl::{self, Unit},
};
use lazy_static::lazy_static;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, MutexGuard};

const MC_SERVER_SERVICE: &str = "mc_server.service";
const REFRESH_RATE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct ServerStatus {
  unit: Option<Unit>,
  last_updated: Instant,
  state: ServerState,
}

impl ServerStatus {
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
    self.state = ServerState::Booting;
  }

  fn begin_shutdown(&mut self) {
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

async fn mc_server_status_guard<'a>(
) -> Result<MutexGuard<'a, ServerStatus>, Box<dyn ThreadSafeError>> {
  lazy_static! {
    static ref SERVER_STATUS: Mutex<ServerStatus> = Mutex::new(ServerStatus::new());
  }
  let mut status_guard = SERVER_STATUS.lock().await;
  status_guard.maybe_update().await?;
  Ok(status_guard)
}

pub async fn mc_server_state() -> Result<ServerState, Box<dyn ThreadSafeError>> {
  Ok(mc_server_status_guard().await?.state())
}

pub async fn boot_server() -> Result<(), Box<dyn ThreadSafeError>> {
  {
    let guard = mc_server_status_guard().await?;
    if guard.state != ServerState::Off {
      return Err(
        McError::InvalidOp(format!("Can't turn server on in {:?} state", guard.state)).into(),
      );
    }
  }

  let exit_status = systemctl::start(MC_SERVER_SERVICE).await?;
  if exit_status.success() {
    let mut guard = mc_server_status_guard().await?;
    guard.begin_boot();
    Ok(())
  } else {
    Err(McError::NonzeroExit(exit_status).into())
  }
}

async fn await_server_shutdown() -> Result<(), Box<dyn ThreadSafeError>> {
  let exit_status = systemctl::stop(MC_SERVER_SERVICE).await?;
  if exit_status.success() {
    mc_server_status_guard().await?.complete_shutdown();
  } else {
    mc_server_status_guard().await?.abort_shutdown();
  }
  Ok(())
}

pub async fn shutdown_server() -> Result<(), Box<dyn ThreadSafeError>> {
  let mut guard = mc_server_status_guard().await?;
  if guard.state != ServerState::On {
    return Err(
      McError::InvalidOp(format!("Can't turn server off in {:?} state", guard.state)).into(),
    );
  }
  tokio::spawn(async {
    if let Err(err) = await_server_shutdown().await {
      eprintln!("Failed to shut down Minecraft Server: {err}");
    }
  });

  guard.begin_shutdown();
  Ok(())
}
