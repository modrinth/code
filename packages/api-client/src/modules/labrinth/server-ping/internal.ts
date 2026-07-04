import { AbstractModule } from '../../../core/abstract-module.js'
import type { Labrinth } from '../types.js'

export class LabrinthServerPingInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_server_ping_internal'
	}

	/**
	 * Ping a Minecraft Java server
	 * POST /_internal/server-ping/minecraft-java
	 */
	public async pingMinecraftJava(
		request: Labrinth.ServerPing.Internal.MinecraftJavaPingRequest,
	): Promise<void> {
		return this.client.request<void>('/server-ping/minecraft-java', {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body: request,
		})
	}
}
