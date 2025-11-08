import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from './types'

export class ArchonServersV0Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_servers_v0'
	}

	/**
	 * Get list of servers for the authenticated user
	 * GET /modrinth/v0/servers
	 */
	public async list(
		options?: Archon.Servers.v0.GetServersOptions,
	): Promise<Archon.Servers.v0.ServerGetResponse> {
		const params = new URLSearchParams()
		if (options?.limit) params.set('limit', options.limit.toString())
		if (options?.offset) params.set('offset', options.offset.toString())

		const query = params.toString() ? `?${params.toString()}` : ''

		return this.client.request<Archon.Servers.v0.ServerGetResponse>(`/modrinth/v0/servers${query}`, {
			api: 'archon',
			method: 'GET',
		})
	}
}
