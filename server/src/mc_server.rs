use crate::{error::McError, proto::ServerState, systemctl::Unit};
use lazy_static::lazy_static;
use std::{
  error,
  time::{Duration, Instant},
};
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
    self.state = ServerState::Booting;
  }

  async fn maybe_update(&mut self) -> Result<(), Box<dyn error::Error>> {
    let now = Instant::now();
    if self.unit.is_some() && now < self.last_updated + REFRESH_RATE {
      return Ok(());
    }
    self.unit = Some(Self::refresh_unit().await?);
    self.last_updated = now;

    self.state = match (self.state, self.unit.as_ref().unwrap().active) {
      (ServerState::Booting, false) => ServerState::Booting,
      (_, false) => ServerState::Off,
      (ServerState::Shutdown, true) => ServerState::Shutdown,
      (_, true) => ServerState::On,
    };
    Ok(())
  }

  async fn refresh_unit() -> Result<Unit, Box<dyn error::Error>> {
    Unit::from_systemctl(MC_SERVER_SERVICE)
      .await
      .map_err(Box::from)
  }
}

async fn mc_server_status_guard<'a>() -> Result<MutexGuard<'a, ServerStatus>, Box<dyn error::Error>>
{
  lazy_static! {
    static ref SERVER_STATUS: Mutex<ServerStatus> = Mutex::new(ServerStatus::new());
  }
  let mut status_guard = SERVER_STATUS.lock().await;
  status_guard.maybe_update().await?;
  Ok(status_guard)
}

pub async fn mc_server_state() -> Result<ServerState, Box<dyn error::Error>> {
  Ok(mc_server_status_guard().await?.state())
}

pub async fn boot_server() -> Result<(), Box<dyn error::Error>> {
  let mut guard = mc_server_status_guard().await?;
  let exit_status = guard.unit.as_ref().unwrap().start().await?;
  if exit_status.success() {
    guard.begin_boot();
    Ok(())
  } else {
    Err(McError::NonzeroExit(exit_status).into())
  }
}
