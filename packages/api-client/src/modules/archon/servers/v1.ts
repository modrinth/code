import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonServersV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_servers_v1'
	}

	/**
	 * Get full server details including worlds, backups, and content
	 * GET /v1/servers/:server_id
	 */
	public async get(serverId: string): Promise<Archon.Servers.v1.ServerFull> {
		return this.client.request<Archon.Servers.v1.ServerFull>(`/servers/${serverId}`, {
			api: 'archon',
			version: 1,
			method: 'GET',
		})
	}

	/**
	 * Get available regions
	 * GET /v1/regions
	 */
	public async getRegions(): Promise<Archon.Servers.v1.Region[]> {
		return this.client.request<Archon.Servers.v1.Region[]>('/regions', {
			api: 'archon',
			version: 1,
			method: 'GET',
		})
	}
}
