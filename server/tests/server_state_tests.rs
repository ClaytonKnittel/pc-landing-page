use std::time::Duration;

use futures_util::Future;
use pc_landing_page::proto::ServerState;
use rstest::{fixture, rstest};
use tokio::{
  join,
  time::{self, Instant},
};

use self::fixtures::Fixture;

mod fixtures {
  use pc_landing_page::{
    controller::ServerController,
    systemctl::{sim_unit::SimUnit, unit::Unit},
  };

  pub struct Fixture {
    controller: ServerController<SimUnit>,
  }

  impl Fixture {
    pub fn new() -> Self {
      Self {
        controller: ServerController::new(SimUnit::new("test_unit.service".to_owned())),
      }
    }

    pub fn controller(&self) -> &ServerController<impl Unit> {
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
async fn test_shutdown_awaits_completion(shutdown_test: impl Future<Output = Fixture>) {
  let shutdown_test = shutdown_test.await;

  let start = Instant::now();
  assert!(shutdown_test.controller().shutdown_server().await.is_ok());
  let end = Instant::now();

  assert!(end - start >= Duration::from_secs(5));
}

#[rstest]
#[tokio::test]
async fn test_shutdown_moves_to_shutdown_state(shutdown_test: impl Future<Output = Fixture>) {
  let shutdown_test = shutdown_test.await;
  let (state, shutdown_result) = join!(
    async {
      time::sleep(Duration::from_millis(1)).await;
      shutdown_test.controller().server_state().await
    },
    shutdown_test.controller().shutdown_server(),
  );
  assert!(shutdown_result.is_ok());
  assert_eq!(state.unwrap(), ServerState::Shutdown);
}

#[rstest]
#[tokio::test]
async fn test_shutdown_stays_shutting_down_for_5s(shutdown_test: impl Future<Output = Fixture>) {
  let shutdown_test = shutdown_test.await;
  let (state, shutdown_result) = join!(
    async {
      time::sleep(Duration::from_millis(4990)).await;
      shutdown_test.controller().server_state().await
    },
    shutdown_test.controller().shutdown_server(),
  );
  assert!(shutdown_result.is_ok());
  assert_eq!(state.unwrap(), ServerState::Shutdown);
}

#[rstest]
#[tokio::test]
async fn test_shutdown_completes_after_5s(shutdown_test: impl Future<Output = Fixture>) {
  let shutdown_test = shutdown_test.await;
  let (state, shutdown_result) = join!(
    async {
      time::sleep(Duration::from_millis(5001)).await;
      shutdown_test.controller().server_state().await
    },
    shutdown_test.controller().shutdown_server(),
  );
  assert!(shutdown_result.is_ok());
  assert_eq!(state.unwrap(), ServerState::Off);
}

#[rstest]
#[tokio::test]
async fn test_shutdown_fails_if_already_shutting_down(
  shutdown_test: impl Future<Output = Fixture>,
) {
  let shutdown_test = shutdown_test.await;
  let shutdown_results = join!(shutdown_test.controller().shutdown_server(), async {
    time::sleep(Duration::from_millis(1)).await;
    shutdown_test.controller().shutdown_server().await
  },);
  assert!(shutdown_results.0.is_ok());
  assert!(shutdown_results.1.is_err());
}

#[rstest]
#[tokio::test]
async fn test_shutdown_fails_if_already_off(shutdown_test: impl Future<Output = Fixture>) {
  let shutdown_test = shutdown_test.await;
  assert!(shutdown_test.controller().shutdown_server().await.is_ok());
  assert!(shutdown_test.controller().shutdown_server().await.is_err());
}
