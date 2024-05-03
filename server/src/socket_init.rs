use std::{net::SocketAddr, sync::Arc, time::Duration};

use async_sockets::{
  AsyncSocket, AsyncSocketContext, AsyncSocketEmitters, AsyncSocketListeners, AsyncSocketOptions,
  AsyncSocketResponders, AsyncSocketSecurity, Status,
};
use serde::Deserialize;
use tokio::task::JoinHandle;

use crate::{
  controller::{
    controller_interface::ServerController, sim_server_controller::SimServerController,
    systemctl_server_controller::SystemctlServerController,
  },
  proto::ServerState,
  security::{CERTFILE, KEYFILE},
};

struct Globals {
  server_controller: Box<dyn ServerController + Send + Sync>,
}

impl Globals {
  fn new(sim: bool) -> Self {
    let server_controller = if sim {
      Box::new(SimServerController::new()) as Box<dyn ServerController + Send + Sync>
    } else {
      Box::new(SystemctlServerController::new())
    };

    Self { server_controller }
  }
}

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
  BootServer {},
  ShutdownServer {},
}

#[derive(AsyncSocketResponders)]
enum ToClientResponses {
  McServerStatus { state: ServerState },
  BootServer {},
  ShutdownServer {},
}

async fn handle_connect_event(_context: AsyncSocketContext<ServerEmitEvents>) {}

async fn handle_call_event(
  event: FromClientRequests,
  _context: AsyncSocketContext<ServerEmitEvents>,
  globals: Arc<Globals>,
) -> Status<ToClientResponses> {
  match event {
    FromClientRequests::McServerStatus {} => match globals.server_controller.server_state().await {
      Ok(state) => Status::Ok(ToClientResponses::McServerStatus { state }),
      Err(_) => Status::InternalServerError("Failed to read MC server status".into()),
    },
    FromClientRequests::BootServer {} => match globals.server_controller.boot_server().await {
      Ok(()) => Status::Ok(ToClientResponses::BootServer {}),
      Err(err) => Status::InternalServerError(format!("Failed to boot server: {err}")),
    },
    FromClientRequests::ShutdownServer {} => {
      match globals.server_controller.shutdown_server().await {
        Ok(()) => Status::Ok(ToClientResponses::ShutdownServer {}),
        Err(err) => Status::InternalServerError(format!("Failed to boot server: {err}")),
      }
    }
  }
}

async fn handle_emit_event(
  event: ClientEmitEvents,
  _context: AsyncSocketContext<ServerEmitEvents>,
) {
  match event {}
}

pub fn create_socket_endpoint(prod: bool, addr: SocketAddr, sim: bool) -> JoinHandle<()> {
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

  let globals = Arc::new(Globals::new(sim));

  tokio::spawn(async move {
    println!(
      "Starting server on {}://{addr}",
      if prod { "wss" } else { "ws" }
    );
    AsyncSocket::new(
      options,
      handle_connect_event,
      handle_emit_event,
      move |event, context| handle_call_event(event, context, globals.clone()),
    )
    .start_server()
    .await
  })
}
