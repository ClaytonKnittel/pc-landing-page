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

  #[arg(long, default_value_t = 2345)]
  ws_port: u16,

  #[arg(long, default_value_t = false)]
  prod: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let args = Args::parse();

  match tokio::join!(
    run_file_server(args.prod, args.port),
    create_socket_endpoint(args.prod, args.ws_port)
  ) {
    (Err(err), _) | (_, Err(err)) => Err(err.into()),
    (Ok(()), Ok(())) => Ok(()),
  }
}
