use std::net::SocketAddr;

use hyper::{
  body::{Body, Incoming},
  server::conn::http1,
  service::Service,
  Request, Response,
};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

pub async fn run_http_service<S, B>(
  service: S,
  addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
  S: Service<Request<Incoming>, Response = Response<B>> + Copy + Send + 'static,
  S::Future: Send + 'static,
  S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
  B: Body + Send + 'static,
  B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
  B::Data: Send,
{
  println!("Starting server on http://{addr}");

  let listener = TcpListener::bind(addr).await?;

  loop {
    let (stream, _) = listener.accept().await?;

    let io = TokioIo::new(stream);

    tokio::task::spawn(async move {
      if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
        eprintln!("Error serving connection: {err:?}");
      }
    });
  }
}
