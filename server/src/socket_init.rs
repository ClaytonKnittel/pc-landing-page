use std::time::Duration;

use async_sockets::{
  AsyncSocket, AsyncSocketContext, AsyncSocketEmitters, AsyncSocketListeners, AsyncSocketOptions,
  AsyncSocketResponders, Status,
};
use serde::Deserialize;
use tokio::task::JoinHandle;

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
  static mut CNT: u64 = 0;
  match event {
    FromClientRequests::McServerStatus {} => {
      unsafe {
        CNT += 1;
      }
      Status::Ok(ToClientResponses::McServerStatus {
        on: unsafe { CNT } % 2 == 0,
      })
    }
  }
}

async fn handle_emit_event(
  event: ClientEmitEvents,
  _context: AsyncSocketContext<ServerEmitEvents>,
) {
  match event {}
}

pub fn create_socket_endpoint() -> JoinHandle<()> {
  tokio::spawn(async {
    AsyncSocket::new(
      AsyncSocketOptions::new()
        .with_path("horsney")
        .with_port(2345)
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
