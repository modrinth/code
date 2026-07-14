import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonPropertiesV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_properties_v1'
	}

	/** GET /v1/servers/:server_id/worlds/:world_id/properties */
	public async getProperties(
		serverId: string,
		worldId: string,
	): Promise<Archon.Content.v1.PropertiesFields> {
		return this.client.request<Archon.Content.v1.PropertiesFields>(
			`/servers/${serverId}/worlds/${worldId}/properties`,
			{
				api: 'archon',
				version: 1,
				method: 'GET',
			},
		)
	}

	/** PATCH /v1/servers/:server_id/worlds/:world_id/properties */
	public async patchProperties(
		serverId: string,
		worldId: string,
		body: Archon.Content.v1.PatchPropertiesFields,
	): Promise<Archon.Content.v1.PropertiesFields> {
		return this.client.request<Archon.Content.v1.PropertiesFields>(
			`/servers/${serverId}/worlds/${worldId}/properties`,
			{
				api: 'archon',
				version: 1,
				method: 'PATCH',
				body,
			},
		)
	}
}
