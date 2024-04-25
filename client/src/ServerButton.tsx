import React from 'react';

import { ServerSocket } from 'client/ServerMsgs';
import { AsyncSocketContext } from 'client/util/async_sockets';
import { isOk } from 'client/util/status';

// TODO: need to use wss for prod
const socket: ServerSocket = new AsyncSocketContext(
  `wss://${window.location.hostname}:2345/horsney`,
  true
);

async function getMcServerStatus(): Promise<boolean> {
  await socket.awaitOpen();
  const status = await socket.call('mc_server_status');
  return isOk(status) && status.value.on;
}

export function ServerButton() {
  const [serverOn, setServerOn] = React.useState(false);
  const setServerOnRef = React.useRef(setServerOn);
  setServerOnRef.current = setServerOn;

  getMcServerStatus().then((isOn) => {
    setServerOnRef.current(isOn);
  });

  if (serverOn) {
    return (
      <div
        onClick={() => {
          setServerOn(false);
        }}
      >
        Turn Server Off
      </div>
    );
  } else {
    return (
      <div
        onClick={() => {
          setServerOn(true);
        }}
      >
        Turn Server On
      </div>
    );
  }
}
