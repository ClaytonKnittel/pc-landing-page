use crate::{error::ThreadSafeError, proto::ServerState};
use async_trait::async_trait;

#[async_trait]
pub trait ServerController {
  /// Returns the current state of the Minecraft server.
  async fn server_state(&self) -> Result<ServerState, Box<dyn ThreadSafeError>>;

  /// Triggers a server boot, which will attempt to turn the Minecraft server
  /// on. Returns upon boot triggering.
  async fn boot_server(&self) -> Result<(), Box<dyn ThreadSafeError>>;

  /// Triggers a server shutdown, which will attempt to turn off the Minecraft
  /// server. Returns upon shutdown completing, which may take a while.
  async fn shutdown_server(&self) -> Result<(), Box<dyn ThreadSafeError>>;
}
