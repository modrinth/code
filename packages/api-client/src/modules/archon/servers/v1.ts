import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonServersV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_servers_v1'
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
