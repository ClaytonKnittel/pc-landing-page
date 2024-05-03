use crate::{error::ThreadSafeError, proto::ServerState};
use async_trait::async_trait;

use super::controller_interface::ServerController;

pub struct SimServerController {
  state: ServerState,
}

impl SimServerController {
  pub fn new() -> Self {
    Self {
      state: ServerState::Off,
    }
  }
}

#[async_trait]
impl ServerController for SimServerController {
  async fn server_state(&self) -> Result<ServerState, Box<dyn ThreadSafeError>> {
    todo!();
  }

  async fn boot_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    todo!();
  }

  async fn shutdown_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    todo!();
  }
}
