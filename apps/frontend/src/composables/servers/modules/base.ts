import type { ModrinthServer } from '../modrinth-servers.ts'

export abstract class ServerModule {
	protected server: ModrinthServer

	constructor(server: ModrinthServer) {
		this.server = server
	}

	protected get serverId(): string {
		return this.server.serverId
	}

	abstract fetch(): Promise<void>
}
