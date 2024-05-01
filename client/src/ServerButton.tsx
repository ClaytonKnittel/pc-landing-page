import React from 'react';

import { ServerSocket } from 'client/ServerMsgs';
import { isOk } from 'client/util/status';
import { ServerState } from 'proto/mc_server';

const SERVER_STATE_REFRESH_INTERVAL = 5000;

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
  const [state, setState] = React.useState(ServerState.UNKNOWN);
  const stateRef = React.useRef(state);
  stateRef.current = state;
  const setStateRef = React.useRef(setState);
  setStateRef.current = setState;

  React.useEffect(() => {
    getMcServerStatus(props.socket).then(setStateRef.current);

    const refreshStateIntervalId = setInterval(
      () => {
        getMcServerStatus(props.socket).then(setStateRef.current);
      },
      SERVER_STATE_REFRESH_INTERVAL,
      props.socket,
      setStateRef
    );

    return () => {
      clearInterval(refreshStateIntervalId);
    };
  }, []);

  let action;
  switch (state) {
    case ServerState.UNKNOWN: {
      action = '';
      break;
    }
    case ServerState.OFF: {
      action = 'Turn Server On';
      break;
    }
    case ServerState.BOOTING: {
      action = 'Turning Server On...';
      break;
    }
    case ServerState.ON: {
      action = 'Turn Server Off';
      break;
    }
    case ServerState.SHUTDOWN: {
      action = 'Turning Server Off...';
      break;
    }
  }

  return (
    <>
      <div
        onClick={() => {
          if (state === ServerState.OFF) {
            props.socket.call('boot_server').then((status) => {
              if (isOk(status)) {
                if (stateRef.current === ServerState.OFF) {
                  setStateRef.current(ServerState.BOOTING);
                }
              }
            });
          } else if (state === ServerState.ON) {
            props.socket.call('shutdown_server').then((status) => {
              if (isOk(status)) {
                if (stateRef.current === ServerState.ON) {
                  setStateRef.current(ServerState.SHUTDOWN);
                }
              }
            });
          }
        }}
      >
        {action}
      </div>
      <br />
      <div>Current Status: {state}</div>
    </>
  );
}
