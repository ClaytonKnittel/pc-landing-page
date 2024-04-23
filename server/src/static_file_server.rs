use http_body_util::Full;
use hyper::{
  body::{self, Bytes},
  service::service_fn,
  Request, Response,
};

use crate::https::run_https_service;

async fn test(_: Request<body::Incoming>) -> hyper::Result<Response<Full<Bytes>>> {
  Ok(Response::new(Full::new("Hello".into())))
}

pub async fn run_https_file_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let service = service_fn(test);
  run_https_service(service).await
}
