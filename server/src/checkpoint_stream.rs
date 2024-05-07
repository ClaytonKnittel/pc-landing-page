use std::{error, ops::DerefMut, time::Duration};

use async_bincode::tokio::AsyncBincodeStream;
use bincode::{DefaultOptions, Serializer};
use futures_util::{SinkExt, Stream, StreamExt};
use serde::Serialize;
use tokio::{
  io::AsyncWrite,
  sync::Mutex,
  task::JoinHandle,
  time::{interval, MissedTickBehavior},
};

pub trait IncrementalUpdate: Serialize {
  /// Should return true when there is new uncommitted incremental state to be
  /// checkpointed.
  fn has_update(&self) -> bool;

  /// Called after the incremental state has been checkpointed. This should
  /// commit all incremental state to the full state.
  fn committed(&mut self);
}

pub struct CheckpointStreamOptions {
  /// The amount of time to wait between successive incremental checkpoints.
  pub poll_period: Duration,
}

impl CheckpointStreamOptions {
  pub fn build<W, S>(self, checkpoint_writer: W, state: Mutex<S>) -> CheckpointStream<W, S>
  where
    W: AsyncWrite + Send + 'static,
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
  W: AsyncWrite + Send + 'static,
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
  pub fn start(mut self) -> JoinHandle<()> {
    tokio::spawn(async move {
      let mut timer = interval(self.options.poll_period);
      timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

      loop {
        timer.tick().await;
        self.tick().await;
      }
    })
  }

  async fn tick(&mut self) -> Result<(), Box<dyn error::Error>> {
    let mut guard = self.state.lock().await;
    let state = guard.deref_mut();

    if state.has_update() {
      // bincode::serialize_into(&mut self.checkpoint_writer, todo!())?;
      let mut serializer = Serializer::new(&mut self.checkpoint_writer, DefaultOptions::default());
      serde::Serialize::serialize(value, &mut serializer);
    }

    Ok(())
  }
}
