import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonServersInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'archon_servers_internal'
	}

	/**
	 * Get the server assigned to a subdomain.
	 * GET /_internal/servers/by-subdomain/:subdomain
	 */
	public async getBySubdomain(subdomain: string): Promise<Archon.Servers.Internal.Lookup> {
		return this.client.request<Archon.Servers.Internal.Lookup>(
			`/servers/by-subdomain/${encodeURIComponent(subdomain)}`,
			{
				api: 'archon',
				version: 'internal',
				method: 'GET',
			},
		)
	}
}
