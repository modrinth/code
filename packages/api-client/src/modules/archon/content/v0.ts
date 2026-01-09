import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonContentV0Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_content_v0'
	}

	/** GET /modrinth/v0/servers/:server_id/mods */
	public async list(serverId: string): Promise<Archon.Content.v0.Mod[]> {
		return this.client.request<Archon.Content.v0.Mod[]>(`/servers/${serverId}/mods`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'GET',
		})
	}

	/** POST /modrinth/v0/servers/:server_id/mods */
	public async install(
		serverId: string,
		request: Archon.Content.v0.InstallModRequest,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/mods`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'POST',
			body: request,
		})
	}

	/** POST /modrinth/v0/servers/:server_id/deleteMod */
	public async delete(
		serverId: string,
		request: Archon.Content.v0.DeleteModRequest,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/deleteMod`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'POST',
			body: request,
		})
	}

	/** POST /modrinth/v0/servers/:server_id/mods/update */
	public async update(
		serverId: string,
		request: Archon.Content.v0.UpdateModRequest,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/mods/update`, {
			api: 'archon',
			version: 'modrinth/v0',
			method: 'POST',
			body: request,
		})
	}
}
