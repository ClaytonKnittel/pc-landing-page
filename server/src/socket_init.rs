use std::time::Duration;

use async_sockets::{
  AsyncSocket, AsyncSocketContext, AsyncSocketEmitters, AsyncSocketListeners, AsyncSocketOptions,
  AsyncSocketResponders, Status,
};
use serde::Deserialize;
use tokio::task::JoinHandle;

use crate::mc_server::mc_server_status;

#[derive(AsyncSocketEmitters)]
enum ServerEmitEvents {}

#[derive(AsyncSocketListeners)]
enum ClientEmitEvents {}

#[derive(AsyncSocketEmitters)]
enum ToClientRequests {}

#[derive(Deserialize)]
enum FromClientResponses {}

#[derive(AsyncSocketListeners)]
enum FromClientRequests {
  McServerStatus {},
}

#[derive(AsyncSocketResponders)]
enum ToClientResponses {
  McServerStatus { on: bool },
}

async fn handle_connect_event(_context: AsyncSocketContext<ServerEmitEvents>) {}

async fn handle_call_event(
  event: FromClientRequests,
  _context: AsyncSocketContext<ServerEmitEvents>,
) -> Status<ToClientResponses> {
  match event {
    FromClientRequests::McServerStatus {} => match mc_server_status() {
      Ok(unit) => Status::Ok(ToClientResponses::McServerStatus { on: unit.active }),
      Err(_) => Status::InternalServerError("Failed to read MC server status".into()),
    },
  }
}

async fn handle_emit_event(
  event: ClientEmitEvents,
  _context: AsyncSocketContext<ServerEmitEvents>,
) {
  match event {}
}

pub fn create_socket_endpoint(prod: bool, port: u16) -> JoinHandle<()> {
  tokio::spawn(async move {
    AsyncSocket::new(
      AsyncSocketOptions::new()
        .with_path("horsney")
        .with_port(port)
        .with_timeout(Duration::from_secs(10))
        .with_verbose(false),
      handle_connect_event,
      handle_emit_event,
      handle_call_event,
    )
    .start_server()
    .await
  })
}
