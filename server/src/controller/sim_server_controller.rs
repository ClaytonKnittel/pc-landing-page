use crate::{
  error::{McError, ThreadSafeError},
  proto::ServerState,
};
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
    Ok(self.state)
  }

  async fn boot_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    Err(McError::InvalidOp("Unimplemented!".to_string()).into())
  }

  async fn shutdown_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    Err(McError::InvalidOp("Unimplemented!".to_string()).into())
  }
}
