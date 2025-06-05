import type { PyroServer } from "../pyro-servers.ts";

export abstract class ServerModule {
  protected server: PyroServer;

  constructor(server: PyroServer) {
    this.server = server;
  }

  protected get serverId(): string {
    return this.server.serverId;
  }

  abstract fetch(): Promise<void>;
}
