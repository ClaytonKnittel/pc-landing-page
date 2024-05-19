use std::{ops::DerefMut, time::Duration};

use serde::Serialize;
use tokio::{
  io::{AsyncWrite, AsyncWriteExt},
  sync::Mutex,
  task::JoinHandle,
  time::{interval, MissedTickBehavior},
};

use crate::error::ThreadSafeError;

pub trait IncrementalUpdate: Serialize {
  /// Should return true when there is new uncommitted incremental state to be
  /// checkpointed.
  fn has_update(&self) -> bool;

  /// Called after the incremental state has been checkpointed. This should
  /// commit all incremental state to the full state.
  fn commit(&mut self) -> Result<Vec<u8>, Box<dyn ThreadSafeError>>;

  /// Don't love this API, think of better way (want to ser/de here, not in
  /// impls).
  fn recover(&mut self, increment_encoding: Vec<u8>) -> Result<(), Box<dyn ThreadSafeError>>;
}

pub struct CheckpointStreamOptions {
  /// The amount of time to wait between successive incremental checkpoints.
  pub poll_period: Duration,
}

impl CheckpointStreamOptions {
  pub fn build<W, S>(self, checkpoint_writer: W, state: Mutex<S>) -> CheckpointStream<W, S>
  where
    W: AsyncWrite + Unpin + Send + 'static,
    S: IncrementalUpdate + Send + Sync + 'static,
  {
    CheckpointStream::from_options(checkpoint_writer, state, self)
  }
}

impl Default for CheckpointStreamOptions {
  fn default() -> Self {
    Self {
      poll_period: Duration::from_secs(30),
    }
  }
}

pub struct CheckpointStream<W, S> {
  checkpoint_writer: W,
  state: Mutex<S>,
  options: CheckpointStreamOptions,
}

impl<W, S> CheckpointStream<W, S>
where
  W: AsyncWriteExt + Unpin + Send + 'static,
  S: IncrementalUpdate + Send + Sync + 'static,
{
  pub fn new(checkpoint_writer: W, state: Mutex<S>) -> Self {
    Self::from_options(checkpoint_writer, state, CheckpointStreamOptions::default())
  }

  fn from_options(checkpoint_writer: W, state: Mutex<S>, options: CheckpointStreamOptions) -> Self {
    Self {
      checkpoint_writer,
      state,
      options,
    }
  }

  /// Starts a separate thread to run the checkpoint stream in. This will
  /// periodically poll `state`, and update the checkpointed state whenever
  /// there is a change.
  pub fn start(mut self) -> JoinHandle<Result<(), Box<dyn ThreadSafeError>>> {
    tokio::spawn(async move {
      let mut timer = interval(self.options.poll_period);
      timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

      loop {
        timer.tick().await;
        self.tick().await?;
      }
    })
  }

  async fn tick(&mut self) -> Result<(), Box<dyn ThreadSafeError>> {
    let mut guard = self.state.lock().await;
    let state = guard.deref_mut();

    if state.has_update() {
      // let mut encoding = bincode::serialize(&state)?;
      let mut encoding = state.commit()?;
      self.checkpoint_writer.write_all(&mut encoding).await?;
      self.checkpoint_writer.flush().await?;
    }

    Ok(())
  }
}
