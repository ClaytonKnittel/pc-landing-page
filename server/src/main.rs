use std::{
  net::{IpAddr, SocketAddr},
  str::FromStr,
};

use clap::Parser;
use pc_landing_page::{
  error::ThreadSafeError, socket_init::create_socket_endpoint, static_file_server::run_file_server,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  #[arg(long, default_value_t = 3000)]
  port: u16,

  #[arg(long)]
  addr: Option<String>,

  #[arg(long, default_value_t = 2345)]
  ws_port: u16,

  /// When passed to the program, runs on https://.
  #[arg(long, default_value_t = false)]
  prod: bool,

  /// When passed to the program, serves production client code, as opposed to
  /// the debug build.
  #[arg(long, default_value_t = false)]
  client_prod: bool,

  /// When passed to the program, runs a simulated Minecraft server instead of
  /// running the systemctl service.
  #[arg(long, default_value_t = false)]
  simulated: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn ThreadSafeError>> {
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
    run_file_server(fs_addr, args.prod, args.client_prod),
    create_socket_endpoint(args.prod, ws_addr, args.simulated).await?
  ) {
    (Err(err), _) | (_, Err(err)) => Err(err.into()),
    (Ok(()), Ok(())) => Ok(()),
  }
}
