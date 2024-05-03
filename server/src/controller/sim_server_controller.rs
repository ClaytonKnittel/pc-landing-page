use std::time::Duration;

use crate::{
  error::{McError, ThreadSafeError},
  proto::ServerState,
};
use async_trait::async_trait;
use tokio::{
  sync::{Mutex, MutexGuard},
  time::Instant,
};

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

  fn maybe_update_state(guard: &mut MutexGuard<'_, SimState>, now: Instant) {
    if guard.state == ServerState::Booting && now >= guard.last_update + OP_DELAY {
      guard.state = ServerState::On;
    } else if guard.state == ServerState::Shutdown && now >= guard.last_update + OP_DELAY {
      guard.state = ServerState::Off;
    }
  }
}

#[async_trait]
impl ServerController for SimServerController {
  async fn server_state(&self) -> Result<ServerState, Box<dyn ThreadSafeError>> {
    let mut guard = self.state.lock().await;
    let now = Instant::now();
    Self::maybe_update_state(&mut guard, now);
    Ok(guard.state)
  }

  async fn boot_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    let mut guard = self.state.lock().await;
    let now = Instant::now();
    Self::maybe_update_state(&mut guard, now);
    if guard.state != ServerState::Off {
      return Err(
        McError::InvalidOp(format!("Server is not in Off state: {:?}", guard.state)).into(),
      );
    }
    guard.state = ServerState::Booting;
    guard.last_update = now;

    Ok(())
  }

  async fn shutdown_server(&self) -> Result<(), Box<dyn ThreadSafeError>> {
    let mut guard = self.state.lock().await;
    let now = Instant::now();
    Self::maybe_update_state(&mut guard, now);
    if guard.state != ServerState::On {
      return Err(
        McError::InvalidOp(format!("Server is not in On state: {:?}", guard.state)).into(),
      );
    }
    guard.state = ServerState::Shutdown;
    guard.last_update = now;

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use std::time::Duration;

  use futures_util::Future;
  use rstest::{fixture, rstest};
  use tokio::time::{self};

  use crate::{controller::controller_interface::ServerController, proto::ServerState};

  use self::fixtures::Fixture;

  mod fixtures {
    use crate::controller::{
      controller_interface::ServerController, sim_server_controller::SimServerController,
    };

    pub struct Fixture {
      controller: SimServerController,
    }

    impl Fixture {
      pub fn new() -> Self {
        Self {
          controller: SimServerController::new(),
        }
      }

      pub fn controller(&self) -> &impl ServerController {
        &self.controller
      }
    }
  }

  #[fixture]
  fn boot_test() -> Fixture {
    time::pause();
    Fixture::new()
  }

  #[rstest]
  #[tokio::test]
  async fn test_default_state(boot_test: Fixture) {
    assert_eq!(
      boot_test.controller().server_state().await.unwrap(),
      ServerState::Off
    );
  }

  #[rstest]
  #[tokio::test]
  async fn test_boot_moves_to_booting_state(boot_test: Fixture) {
    assert!(boot_test.controller().boot_server().await.is_ok());
    assert_eq!(
      boot_test.controller().server_state().await.unwrap(),
      ServerState::Booting
    );
  }

  #[rstest]
  #[tokio::test]
  async fn test_boot_stays_booting_for_5s(boot_test: Fixture) {
    assert!(boot_test.controller().boot_server().await.is_ok());
    time::sleep(Duration::from_millis(4990)).await;
    assert_eq!(
      boot_test.controller().server_state().await.unwrap(),
      ServerState::Booting
    );
  }

  #[rstest]
  #[tokio::test]
  async fn test_boot_completes_after_5s(boot_test: Fixture) {
    assert!(boot_test.controller().boot_server().await.is_ok());
    time::sleep(Duration::from_millis(5001)).await;
    assert_eq!(
      boot_test.controller().server_state().await.unwrap(),
      ServerState::On
    );
  }

  #[rstest]
  #[tokio::test]
  async fn test_boot_fails_if_already_booting(boot_test: Fixture) {
    assert!(boot_test.controller().boot_server().await.is_ok());
    assert!(boot_test.controller().boot_server().await.is_err());
  }

  #[rstest]
  #[tokio::test]
  async fn test_boot_fails_if_already_on(boot_test: Fixture) {
    assert!(boot_test.controller().boot_server().await.is_ok());
    time::sleep(Duration::from_secs(6)).await;
    assert!(boot_test.controller().boot_server().await.is_err());
  }

  #[fixture]
  async fn shutdown_test() -> Fixture {
    time::pause();
    let fixture = Fixture::new();
    fixture.controller().boot_server().await.unwrap();
    time::sleep(Duration::from_secs(6)).await;
    fixture
  }

  #[rstest]
  #[tokio::test]
  async fn test_shutdown_moves_to_shutdown_state(shutdown_test: impl Future<Output = Fixture>) {
    let shutdown_test = shutdown_test.await;
    assert!(shutdown_test.controller().shutdown_server().await.is_ok());
    assert_eq!(
      shutdown_test.controller().server_state().await.unwrap(),
      ServerState::Shutdown
    );
  }

  #[rstest]
  #[tokio::test]
  async fn test_shutdown_stays_shutting_down_for_5s(shutdown_test: impl Future<Output = Fixture>) {
    let shutdown_test = shutdown_test.await;
    assert!(shutdown_test.controller().shutdown_server().await.is_ok());
    time::sleep(Duration::from_millis(4990)).await;
    assert_eq!(
      shutdown_test.controller().server_state().await.unwrap(),
      ServerState::Shutdown
    );
  }

  #[rstest]
  #[tokio::test]
  async fn test_shutdown_completes_after_5s(shutdown_test: impl Future<Output = Fixture>) {
    let shutdown_test = shutdown_test.await;
    assert!(shutdown_test.controller().shutdown_server().await.is_ok());
    time::sleep(Duration::from_millis(5001)).await;
    assert_eq!(
      shutdown_test.controller().server_state().await.unwrap(),
      ServerState::Off
    );
  }

  #[rstest]
  #[tokio::test]
  async fn test_shutdown_fails_if_already_shutting_down(
    shutdown_test: impl Future<Output = Fixture>,
  ) {
    let shutdown_test = shutdown_test.await;
    assert!(shutdown_test.controller().shutdown_server().await.is_ok());
    assert!(shutdown_test.controller().shutdown_server().await.is_err());
  }

  #[rstest]
  #[tokio::test]
  async fn test_shutdown_fails_if_already_off(shutdown_test: impl Future<Output = Fixture>) {
    let shutdown_test = shutdown_test.await;
    assert!(shutdown_test.controller().shutdown_server().await.is_ok());
    time::sleep(Duration::from_secs(6)).await;
    assert!(shutdown_test.controller().shutdown_server().await.is_err());
  }
}
