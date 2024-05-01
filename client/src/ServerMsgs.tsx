import { AsyncSocketContext } from 'client/util/async_sockets';
import { Status } from 'client/util/status';
import { Empty } from 'client/util/util';
import { ServerState } from 'proto/mc_server';

interface ServerToClient {
  /* eslint-disable @typescript-eslint/naming-convention */
  boot_server_res: (res: Status<Empty>) => void;
  shutdown_server_res: (res: Status<Empty>) => void;
  mc_server_status_res: (res: Status<{ state: ServerState }>) => void;
  /* eslint-enable @typescript-eslint/naming-convention */
}

interface ClientToServer {
  /* eslint-disable @typescript-eslint/naming-convention */
  boot_server_req: () => void;
  shutdown_server_req: () => void;
  mc_server_status_req: () => void;
  /* eslint-enable @typescript-eslint/naming-convention */
}

export type ServerSocket = AsyncSocketContext<ServerToClient, ClientToServer>;
