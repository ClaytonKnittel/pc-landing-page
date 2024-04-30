import React from 'react';

import { ServerSocket } from 'client/ServerMsgs';
import { isOk } from 'client/util/status';
import { ServerState } from 'proto/mc_server';

async function getMcServerStatus(socket: ServerSocket): Promise<ServerState> {
  await socket.awaitOpen();
  const status = await socket.call('mc_server_status');
  if (isOk(status)) {
    return status.value.state;
  }
  return ServerState.UNKNOWN;
}

export interface ServerButtonProps {
  socket: ServerSocket;
}

export function ServerButton(props: ServerButtonProps) {
  const [serverOn, setServerOn] = React.useState(false);

  const [state, setState] = React.useState(ServerState.UNKNOWN);
  const setStateRef = React.useRef(setState);
  setStateRef.current = setState;

  React.useEffect(() => {
    getMcServerStatus(props.socket).then(setStateRef.current);
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
