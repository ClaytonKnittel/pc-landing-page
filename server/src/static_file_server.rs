use std::{fs, net::SocketAddr};

use tokio::task::JoinHandle;
use warp::Filter;

use crate::security::{CERTFILE, KEYFILE};

const DIR: &str = "../client/dist/dev/static";

pub fn run_file_server(prod: bool, addr: SocketAddr) -> JoinHandle<()> {
  tokio::spawn(async move {
    println!(
      "Starting server on {}{addr}",
      if prod { "https://" } else { "http://" }
    );

    let directory = fs::canonicalize(DIR).unwrap();
    let route = warp::get().and(warp::fs::dir(directory));
    let server = warp::serve(route);
    if prod {
      server
        .tls()
        .cert_path(CERTFILE)
        .key_path(KEYFILE)
        .run(addr)
        .await
    } else {
      server.run(addr).await
    }
  })
}
