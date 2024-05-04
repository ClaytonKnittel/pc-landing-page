use futures_util::Future;

pub const SYSTEMCTL_PATH: &str = "/usr/bin/systemctl";

pub trait ThreadSafeFuture: Future + Send + Sync + 'static {}

impl<T> ThreadSafeFuture for T where T: Future + Send + Sync + 'static {}

impl<T> From<T> for Box<dyn ThreadSafeFuture<Output = T::Output>>
where
  T: ThreadSafeFuture + Sized,
{
  fn from(value: T) -> Self {
    Box::new(value)
  }
}
