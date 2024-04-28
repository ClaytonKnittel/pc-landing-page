use std::{fs, io, net::SocketAddr, path::PathBuf};

use futures_util::TryStreamExt;
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::{
  body::{self, Bytes, Frame},
  service::service_fn,
  Method, Request, Response, StatusCode,
};
use tokio::{fs::File, task::JoinHandle};
use tokio_util::io::ReaderStream;

use crate::{http::run_http_service, https::run_https_service};

const DIR: &str = "../client/dist/dev/static";
const INDEX: &str = "/index.html";

fn client_static_path() -> PathBuf {
  fs::canonicalize(DIR).unwrap()
}

async fn service(
  req: Request<body::Incoming>,
) -> hyper::Result<Response<BoxBody<Bytes, io::Error>>> {
  match (req.method(), req.uri().path()) {
    (&Method::GET, "/") => respond_file_contents(INDEX).await,
    (&Method::GET, uri) => respond_file_contents(uri).await,
    _ => Ok(not_found()),
  }
}

fn not_found() -> Response<BoxBody<Bytes, io::Error>> {
  Response::builder()
    .status(StatusCode::NOT_FOUND)
    .body(
      Full::new("Not Found".into())
        .map_err(|e| match e {})
        .boxed(),
    )
    .unwrap()
}

async fn respond_file_contents(uri: &str) -> hyper::Result<Response<BoxBody<Bytes, io::Error>>> {
  let uri = uri.strip_prefix('/').unwrap_or(uri);
  let full_path = client_static_path().join(uri);
  if !full_path.starts_with(client_static_path()) {
    return Ok(not_found());
  }

  if let Ok(file) = File::open(full_path).await {
    let reader_stream = ReaderStream::new(file);
    let body_stream = StreamBody::new(reader_stream.map_ok(Frame::data)).boxed();
    let response = Response::builder()
      .status(StatusCode::OK)
      .body(body_stream)
      .unwrap_or_else(|_| not_found());
    return Ok(response);
  }

  Ok(not_found())
}

pub fn run_file_server(prod: bool, addr: SocketAddr) -> JoinHandle<()> {
  tokio::spawn(async move {
    let service = service_fn(service);
    if prod {
      run_https_service(service, addr).await.unwrap();
    } else {
      run_http_service(service, addr).await.unwrap();
    }
  })
}
