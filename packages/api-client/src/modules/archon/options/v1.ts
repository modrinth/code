import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonOptionsV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_options_v1'
	}

	/** GET /v1/servers/:server_id/worlds/:world_id/options/startup */
	public async getStartup(
		serverId: string,
		worldId: string,
	): Promise<Archon.Content.v1.RuntimeOptions> {
		return this.client.request<Archon.Content.v1.RuntimeOptions>(
			`/servers/${serverId}/worlds/${worldId}/options/startup`,
			{
				api: 'archon',
				version: 1,
				method: 'GET',
			},
		)
	}

	/** PATCH /v1/servers/:server_id/worlds/:world_id/options/startup */
	public async patchStartup(
		serverId: string,
		worldId: string,
		body: Archon.Content.v1.PatchRuntimeOptions,
	): Promise<void> {
		await this.client.request(`/servers/${serverId}/worlds/${worldId}/options/startup`, {
			api: 'archon',
			version: 1,
			method: 'PATCH',
			body,
		})
	}
}
