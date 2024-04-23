use clap::Parser;
use static_file_server::run_file_server;

mod http;
mod https;
mod static_file_server;
mod util;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  #[arg(long, default_value_t = 3000)]
  port: u16,

  #[arg(long, default_value_t = false)]
  prod: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let args = Args::parse();

  run_file_server(args.prod, args.port).await
}
