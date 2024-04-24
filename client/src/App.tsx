import React from 'react';

import { ServerButton } from './ServerButton';

// import { AsyncSocketContext } from "client/util/async_sockets";
// import { isOk } from "client/util/status";
// import { Test } from "proto/test";

// const socket: OnoroSocket = new AsyncSocketContext(
//   "ws://[::]:2345/onoro",
//   true,
// );

export function App() {
  return <ServerButton />;
}
