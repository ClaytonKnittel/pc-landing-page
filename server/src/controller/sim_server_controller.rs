use std::time::Duration;

use crate::{
  error::{McError, ThreadSafeError},
  proto::ServerState,
};
use async_trait::async_trait;
use tokio::{sync::Mutex, time::Instant};

use super::controller_interface::ServerController;

const OP_DELAY: Duration = Duration::from_secs(5);

struct SimState {
  state: ServerState,
  last_update: Instant,
}

pub struct SimServerController {
  state: Mutex<SimState>,
}

impl SimServerController {
  pub fn new() -> Self {
    Self {
      state: SimState {
        state: ServerState::Off,
        last_update: Instant::now(),
      }
      .into(),
    }
  }
}

#[async_trait]
impl ServerController for SimServerController {
  async fn server_state(&self) -> Result<ServerState, Box<dyn ThreadSafeError>> {
    let mut guard = self.state.lock().await;
    let now = Instant::now();
    if guard.state == ServerState::Booting && now >= guard.last_update + OP_DELAY {
      guard.state = ServerState::On;
    } else if guard.state == ServerState::Shutdown && now >= guard.last_update + OP_DELAY {
      guard.state = ServerState::Off;
    }
    Ok(guard.state)
  }

  async fn boot_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    let mut guard = self.state.lock().await;
    if guard.state != ServerState::Off {
      return Err(
        McError::InvalidOp(format!("Server is not in Off state: {:?}", guard.state)).into(),
      );
    }
    guard.state = ServerState::Booting;
    guard.last_update = Instant::now();

    Ok(())
  }

  async fn shutdown_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    let mut guard = self.state.lock().await;
    if guard.state != ServerState::On {
      return Err(
        McError::InvalidOp(format!("Server is not in On state: {:?}", guard.state)).into(),
      );
    }
    guard.state = ServerState::Shutdown;
    guard.last_update = Instant::now();

    Ok(())
  }
}
