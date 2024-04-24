import React from 'react';

import { ServerSocket } from 'client/ServerMsgs';
import { AsyncSocketContext } from 'client/util/async_sockets';
import { isOk } from 'client/util/status';

const socket: ServerSocket = new AsyncSocketContext(
  'ws://[::]:2345/horsney',
  true
);

async function GetMcServerStatus() {
  await socket.awaitOpen();
  socket.call('mc_server_status').then((status) => {
    console.log(status);
    if (isOk(status)) {
      console.log(status.value.on);
    }
  });
}

export function ServerButton() {
  const [serverOn, setServerOn] = React.useState(false);

  console.log('rendering');
  GetMcServerStatus();

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
