use std::{
  net::{IpAddr, SocketAddr},
  str::FromStr,
};

use clap::Parser;
use static_file_server::run_file_server;

use crate::socket_init::create_socket_endpoint;

mod http;
mod https;
mod mc_server;
mod security;
mod socket_init;
mod static_file_server;
mod util;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  #[arg(long, default_value_t = 3000)]
  port: u16,

  #[arg(long)]
  addr: Option<String>,

  #[arg(long, default_value_t = 2345)]
  ws_port: u16,

  #[arg(long, default_value_t = false)]
  prod: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  pretty_env_logger::init();
  let args = Args::parse();

  let addr = match (args.addr, args.prod) {
    (Some(addr), _) => IpAddr::from_str(&addr)?,
    (None, true) => [10, 0, 0, 181].into(),
    (None, false) => [127, 0, 0, 1].into(),
  };

  let fs_addr = SocketAddr::new(addr, args.port);
  let ws_addr = SocketAddr::new(addr, args.ws_port);

  match tokio::join!(
    run_file_server(args.prod, fs_addr),
    create_socket_endpoint(args.prod, ws_addr)
  ) {
    (Err(err), _) | (_, Err(err)) => Err(err.into()),
    (Ok(()), Ok(())) => Ok(()),
  }
}
