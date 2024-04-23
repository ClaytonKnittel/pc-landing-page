use static_file_server::run_https_file_server;

mod https;
mod static_file_server;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  run_https_file_server().await
}
