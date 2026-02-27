import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

/**
 * Default world ID - Uuid::nil() which the backend treats as "first/active world"
 * See: apps/archon/src/routes/v1/servers/worlds/mod.rs - world_id_nullish()
 * TODO:
 * - Make sure world ID is being passed before we ship worlds.
 */
const DEFAULT_WORLD_ID: string = '00000000-0000-0000-0000-000000000000' as const

export class ArchonContentV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_content_v1'
	}

	/** GET /v1/:server_id/worlds/:world_id/addons */
	public async getAddons(
		serverId: string,
		worldId: string = DEFAULT_WORLD_ID,
		options?: {
			from_modpack?: boolean
			disabled?: boolean
			addons?: boolean
			updates?: boolean
		},
	): Promise<Archon.Content.v1.Addons> {
		const params = new URLSearchParams()
		if (options?.from_modpack !== undefined)
			params.set('from_modpack', String(options.from_modpack))
		if (options?.disabled !== undefined) params.set('disabled', String(options.disabled))
		if (options?.addons !== undefined) params.set('addons', String(options.addons))
		if (options?.updates !== undefined) params.set('updates', String(options.updates))
		const query = params.toString()

		return this.client.request<Archon.Content.v1.Addons>(
			`/servers/${serverId}/worlds/${worldId}/addons${query ? `?${query}` : ''}`,
			{
				api: 'archon',
				version: 1,
				method: 'GET',
			},
		)
	}

	/** POST /v1/:server_id/worlds/:world_id/addons */
	public async addAddon(
		serverId: string,
		request: Archon.Content.v1.AddAddonRequest,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/worlds/${worldId}/addons`, {
			api: 'archon',
			version: 1,
			method: 'POST',
			body: request,
		})
	}

	/** POST /v1/:server_id/worlds/:world_id/addons/delete */
	public async deleteAddon(
		serverId: string,
		request: Archon.Content.v1.RemoveAddonRequest,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/worlds/${worldId}/addons/delete`, {
			api: 'archon',
			version: 1,
			method: 'POST',
			body: request,
		})
	}

	/** POST /v1/:server_id/worlds/:world_id/addons/disable */
	public async disableAddon(
		serverId: string,
		request: Archon.Content.v1.RemoveAddonRequest,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/worlds/${worldId}/addons/disable`, {
			api: 'archon',
			version: 1,
			method: 'POST',
			body: request,
		})
	}

	/** POST /v1/:server_id/worlds/:world_id/addons/enable */
	public async enableAddon(
		serverId: string,
		request: Archon.Content.v1.RemoveAddonRequest,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/worlds/${worldId}/addons/enable`, {
			api: 'archon',
			version: 1,
			method: 'POST',
			body: request,
		})
	}

	/** POST /v1/:server_id/worlds/:world_id/addons/delete-many */
	public async deleteAddons(
		serverId: string,
		items: Archon.Content.v1.RemoveAddonRequest[],
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/worlds/${worldId}/addons/delete-many`, {
			api: 'archon',
			version: 1,
			method: 'POST',
			body: { items },
		})
	}

	/** POST /v1/:server_id/worlds/:world_id/addons/disable-many */
	public async disableAddons(
		serverId: string,
		items: Archon.Content.v1.RemoveAddonRequest[],
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/worlds/${worldId}/addons/disable-many`, {
			api: 'archon',
			version: 1,
			method: 'POST',
			body: { items },
		})
	}

	/** POST /v1/:server_id/worlds/:world_id/addons/enable-many */
	public async enableAddons(
		serverId: string,
		items: Archon.Content.v1.RemoveAddonRequest[],
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/worlds/${worldId}/addons/enable-many`, {
			api: 'archon',
			version: 1,
			method: 'POST',
			body: { items },
		})
	}

	/** POST /v1/:server_id/worlds/:world_id/content */
	public async installContent(
		serverId: string,
		request: Archon.Content.v1.InstallWorldContent,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/worlds/${worldId}/content`, {
			api: 'archon',
			version: 1,
			method: 'POST',
			body: request,
		})
	}

	/** POST /v1/:server_id/worlds/:world_id/content/unlink-modpack */
	public async unlinkModpack(serverId: string, worldId: string = DEFAULT_WORLD_ID): Promise<void> {
		await this.client.request<void>(
			`/servers/${serverId}/worlds/${worldId}/content/unlink-modpack`,
			{
				api: 'archon',
				version: 1,
				method: 'POST',
			},
		)
	}

	/** GET /v1/:server_id/worlds/:world_id/addons/update?filename=... */
	public async getAddonUpdate(
		serverId: string,
		filename: string,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<Archon.Content.v1.Addon> {
		return this.client.request<Archon.Content.v1.Addon>(
			`/servers/${serverId}/worlds/${worldId}/addons/update?filename=${encodeURIComponent(filename)}`,
			{
				api: 'archon',
				version: 1,
				method: 'GET',
			},
		)
	}

	/** POST /v1/:server_id/worlds/:world_id/addons/update */
	public async updateAddon(
		serverId: string,
		request: Archon.Content.v1.UpdateAddonRequest,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<void> {
		await this.client.request<void>(`/servers/${serverId}/worlds/${worldId}/addons/update`, {
			api: 'archon',
			version: 1,
			method: 'POST',
			body: request,
		})
	}

	/** GET /v1/:server_id/worlds/:world_id/addons/modpack/update */
	public async getModpackUpdate(
		serverId: string,
		worldId: string = DEFAULT_WORLD_ID,
	): Promise<Archon.Content.v1.ModpackFields> {
		return this.client.request<Archon.Content.v1.ModpackFields>(
			`/servers/${serverId}/worlds/${worldId}/addons/modpack/update`,
			{
				api: 'archon',
				version: 1,
				method: 'GET',
			},
		)
	}

	/** POST /v1/:server_id/worlds/:world_id/addons/modpack/update */
	public async updateModpack(serverId: string, worldId: string = DEFAULT_WORLD_ID): Promise<void> {
		await this.client.request<void>(
			`/servers/${serverId}/worlds/${worldId}/addons/modpack/update`,
			{
				api: 'archon',
				version: 1,
				method: 'POST',
			},
		)
	}
}
