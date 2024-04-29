import React from 'react';

import { ServerSocket } from 'client/ServerMsgs';
import { AsyncSocketContext } from 'client/util/async_sockets';
import { isOk } from 'client/util/status';
import { inSecureEnvironment } from 'client/util/util';

enum ServerState {
  Unknown = 'Unknown',
  Off = 'Off',
  Booting = 'Booting',
  On = 'On',
  ShutDown = 'ShutDown',
}

const socket: ServerSocket = new AsyncSocketContext(
  `${inSecureEnvironment() ? 'wss' : 'ws'}://${
    window.location.hostname
  }:2345/horsney`,
  true
);

async function getMcServerStatus(): Promise<ServerState> {
  await socket.awaitOpen();
  const status = await socket.call('mc_server_status');
  if (isOk(status)) {
    return status.value.on ? ServerState.On : ServerState.Off;
  }
  return ServerState.Unknown;
}

export function ServerButton() {
  const [serverOn, setServerOn] = React.useState(false);
  const setServerOnRef = React.useRef(setServerOn);
  setServerOnRef.current = setServerOn;

  const [state, setState] = React.useState(ServerState.Unknown);

  const setStateRef = React.useRef(setState);
  setStateRef.current = setState;

  React.useEffect(() => {
    getMcServerStatus().then(setStateRef.current);
  }, []);

  return (
    <>
      <div
        onClick={() => {
          setServerOn(!serverOn);
        }}
      >
        Turn Server {serverOn ? 'Off' : 'On'}
      </div>
      <br />
      <div>Current Status: {state}</div>
    </>
  );
}
