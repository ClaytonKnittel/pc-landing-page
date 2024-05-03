import React from 'react';

import { ServerButton } from 'client/ServerButton';
import { ServerSocket } from 'client/ServerMsgs';
import { AsyncSocketContext } from 'client/util/async_sockets';
import { inSecureEnvironment } from 'client/util/util';

const socket: ServerSocket = new AsyncSocketContext(
  `${inSecureEnvironment() ? 'wss' : 'ws'}://${
    window.location.hostname
  }:2345/horsney`,
  true,
  20 * 1000
);

export function App() {
  return <ServerButton socket={socket} />;
}
