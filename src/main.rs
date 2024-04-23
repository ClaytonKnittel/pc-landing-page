use std::net::SocketAddr;

use http_body_util::Full;
use hyper::{
  body::{self, Bytes},
  server::conn::http1,
  service::service_fn,
  Request, Response,
};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

async fn test(_: Request<body::Incoming>) -> hyper::Result<Response<Full<Bytes>>> {
  Ok(Response::new(Full::new("Hello".into())))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

  let listener = TcpListener::bind(addr).await?;

  loop {
    let (stream, _) = listener.accept().await?;

    let io = TokioIo::new(stream);

    tokio::task::spawn(async move {
      if let Err(err) = http1::Builder::new()
        .serve_connection(io, service_fn(test))
        .await
      {
        eprintln!("Error serving connection: {err:?}");
      }
    });
  }
}
