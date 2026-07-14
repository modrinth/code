import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonNodesInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'archon_nodes_internal'
	}

	/**
	 * Get node hostnames and region summary for admin tooling.
	 * GET /_internal/nodes/overview
	 */
	public async overview(): Promise<Archon.Nodes.Internal.Overview> {
		return this.client.request<Archon.Nodes.Internal.Overview>('/nodes/overview', {
			api: 'archon',
			version: 'internal',
			method: 'GET',
		})
	}
}
