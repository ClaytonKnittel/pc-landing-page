use std::{os::unix::process::ExitStatusExt, process::ExitStatus, time::Duration};

use async_trait::async_trait;
use futures_util::future::ready;
use tokio::time::Instant;

use crate::{
  error::{McError, ThreadSafeError},
  proto::ServerState,
};

use super::unit::{AsyncResult, Unit};

const OP_DELAY: Duration = Duration::from_secs(5);

pub struct SimUnit {
  name: String,
  state: ServerState,
  last_update: Instant,
}

impl SimUnit {
  pub fn new(name: String) -> Self {
    Self {
      name,
      state: ServerState::Off,
      last_update: Instant::now(),
    }
  }
}

#[async_trait]
impl Unit for SimUnit {
  fn name(&self) -> &str {
    &self.name
  }

  async fn refresh(&mut self) -> Result<(), Box<dyn ThreadSafeError>> {
    let now = Instant::now();
    if self.state == ServerState::Booting && now >= self.last_update + OP_DELAY {
      self.state = ServerState::On;
    } else if self.state == ServerState::Shutdown && now >= self.last_update + OP_DELAY {
      self.state = ServerState::Off;
    }
    Ok(())
  }

  fn restart(&mut self) -> AsyncResult<ExitStatus> {
    #[allow(unreachable_code)]
    Box::pin(ready(unimplemented!()))
  }

  fn start(&mut self) -> AsyncResult<ExitStatus> {
    if self.state != ServerState::Off {
      return Box::pin(ready(Err(
        McError::InvalidOp(format!("Server is not in Off state: {:?}", self.state)).into(),
      )));
    }
    self.state = ServerState::Booting;
    self.last_update = Instant::now();
    Box::pin(ready(Ok(ExitStatus::from_raw(0))))
  }

  fn stop(&mut self) -> AsyncResult<ExitStatus> {
    if self.state != ServerState::On {
      return Box::pin(ready(Err(
        McError::InvalidOp(format!("Server is not in On state: {:?}", self.state)).into(),
      )));
    }
    self.state = ServerState::Shutdown;
    self.last_update = Instant::now();
    Box::pin(ready(Ok(ExitStatus::from_raw(0))))
  }

  fn reload(&mut self) -> AsyncResult<ExitStatus> {
    #[allow(unreachable_code)]
    Box::pin(ready(unimplemented!()))
  }

  fn reload_or_restart(&mut self) -> AsyncResult<ExitStatus> {
    #[allow(unreachable_code)]
    Box::pin(ready(unimplemented!()))
  }

  fn enable(&mut self) -> AsyncResult<ExitStatus> {
    #[allow(unreachable_code)]
    Box::pin(ready(unimplemented!()))
  }

  fn disable(&mut self) -> AsyncResult<ExitStatus> {
    #[allow(unreachable_code)]
    Box::pin(ready(unimplemented!()))
  }

  fn status(&self) -> AsyncResult<String> {
    #[allow(unreachable_code)]
    Box::pin(ready(unimplemented!()))
  }

  fn is_active(&self) -> bool {
    matches!(self.state, ServerState::On)
  }

  fn isolate(&mut self) -> AsyncResult<ExitStatus> {
    #[allow(unreachable_code)]
    Box::pin(ready(unimplemented!()))
  }

  fn freeze(&mut self) -> AsyncResult<ExitStatus> {
    #[allow(unreachable_code)]
    Box::pin(ready(unimplemented!()))
  }

  fn unfreeze(&mut self) -> AsyncResult<ExitStatus> {
    #[allow(unreachable_code)]
    Box::pin(ready(unimplemented!()))
  }

  fn exists(&self) -> AsyncResult<bool> {
    #[allow(unreachable_code)]
    Box::pin(ready(unimplemented!()))
  }
}
