use std::{fs, net::SocketAddr};

use tokio::task::JoinHandle;
use warp::{
  http::HeaderValue,
  reply::{self, Reply},
  Filter,
};

use crate::security::{CERTFILE, KEYFILE};

const DEV_DIR: &str = "../client/dist/dev/static";
const PROD_DIR: &str = "../client/dist/prod/static";

fn disable_cache(reply: impl Reply) -> impl Reply {
  reply::with_header(reply, "cache_control", HeaderValue::from_static("no-cache"))
}

pub fn run_file_server(addr: SocketAddr, prod: bool, client_prod: bool) -> JoinHandle<()> {
  tokio::spawn(async move {
    println!(
      "Starting server on {}{addr}",
      if prod { "https://" } else { "http://" }
    );

    let directory = fs::canonicalize(if client_prod { PROD_DIR } else { DEV_DIR }).unwrap();
    let route = warp::get().and(warp::fs::dir(directory));
    if prod {
      let server = warp::serve(route);
      server
        .tls()
        .cert_path(CERTFILE)
        .key_path(KEYFILE)
        .run(addr)
        .await
    } else {
      let route = route.map(disable_cache);
      let server = warp::serve(route);
      server.run(addr).await
    }
  })
}
