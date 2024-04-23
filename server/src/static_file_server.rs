use std::{fs, io, path::PathBuf};

use http_body_util::Full;
use hyper::{
  body::{self, Bytes},
  service::service_fn,
  Request, Response,
};

use crate::{http::run_http_service, https::run_https_service};

const DIR: &str = "../client/dist/dev/static";

fn client_static_path() -> io::Result<PathBuf> {
  fs::canonicalize(DIR)
}

async fn test(req: Request<body::Incoming>) -> hyper::Result<Response<Full<Bytes>>> {
  Ok(Response::new(Full::new("Hello".into())))
}

pub async fn run_file_server(
  prod: bool,
  port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let service = service_fn(test);
  if prod {
    run_https_service(service, port).await
  } else {
    run_http_service(service, port).await
  }
}
