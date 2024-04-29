use std::{net::SocketAddr, time::Duration};

use async_sockets::{
  AsyncSocket, AsyncSocketContext, AsyncSocketEmitters, AsyncSocketListeners, AsyncSocketOptions,
  AsyncSocketResponders, AsyncSocketSecurity, Status,
};
use serde::Deserialize;
use tokio::task::JoinHandle;

use crate::{
  mc_server::mc_server_status,
  security::{CERTFILE, KEYFILE},
};

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

pub fn create_socket_endpoint(prod: bool, addr: SocketAddr) -> JoinHandle<()> {
  let options = AsyncSocketOptions::new()
    .with_path("horsney")
    .with_bind_addr(addr)
    .with_timeout(Duration::from_secs(10))
    .with_verbose(false);

  let options = if prod {
    options.with_security(AsyncSocketSecurity {
      cert_path: CERTFILE.to_owned(),
      key_path: KEYFILE.to_owned(),
    })
  } else {
    options
  };

  tokio::spawn(async move {
    println!(
      "Starting server on {}://{addr}",
      if prod { "wss" } else { "ws" }
    );
    AsyncSocket::new(
      options,
      handle_connect_event,
      handle_emit_event,
      handle_call_event,
    )
    .start_server()
    .await
  })
}
