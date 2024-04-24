import { AsyncSocketContext } from 'client/util/async_sockets';
import { Status } from 'client/util/status';

interface ServerToClient {
  /* eslint-disable @typescript-eslint/naming-convention */
  mc_server_status_res: (res: Status<{ on: boolean }>) => void;
  /* eslint-enable @typescript-eslint/naming-convention */
}

interface ClientToServer {
  /* eslint-disable @typescript-eslint/naming-convention */
  mc_server_status_req: () => void;
  /* eslint-enable @typescript-eslint/naming-convention */
}

export type ServerSocket = AsyncSocketContext<ServerToClient, ClientToServer>;
