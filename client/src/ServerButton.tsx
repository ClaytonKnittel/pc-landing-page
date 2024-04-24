import React from 'react';

import { ServerSocket } from './ServerMsgs';
import { AsyncSocketContext } from './util/async_sockets';
import { isOk } from './util/status';

export function ServerButton() {
  const [serverOn, setServerOn] = React.useState(false);
  const socket: ServerSocket = new AsyncSocketContext(
    'ws://[::]:2345/horsney',
    true
  );
  socket.call('mc_server_status').then((status) => {
    console.log(status);
    if (isOk(status)) {
      console.log(status.value.on);
    }
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
