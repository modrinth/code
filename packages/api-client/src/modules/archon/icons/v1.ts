import { AbstractModule } from '../../../core/abstract-module'
import type { IconConfig } from '../../../types/icons'
import type { Archon } from '../types'

export class ArchonIconsV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_icons_v1'
	}

	/**
	 * Get a server's custom icon PNG.
	 * GET /v1/servers/:id/icon
	 *
	 * Returns the 64x64 game icon, or the up to 512x512 display icon when `original` is set.
	 */
	public async get(serverId: string, original = false): Promise<Blob> {
		return this.client.request<Blob>(`/servers/${encodeURIComponent(serverId)}/icon`, {
			api: 'archon',
			version: 1,
			method: 'GET',
			responseType: 'blob',
			params: original ? { original: 'true' } : undefined,
		})
	}

	/**
	 * Get a server's custom icon hash and icon editor config.
	 * GET /v1/servers/:id/icon/metadata
	 *
	 * Responds with 404 when the server has no custom icon.
	 */
	public async getMetadata(serverId: string): Promise<Archon.Icons.v1.ServerIcon> {
		return this.client.request<Archon.Icons.v1.ServerIcon>(
			`/servers/${encodeURIComponent(serverId)}/icon/metadata`,
			{
				api: 'archon',
				version: 1,
				method: 'GET',
			},
		)
	}

	/**
	 * Upload an image as a server's icon, clearing any icon editor config.
	 * PUT /v1/servers/:id/icon
	 */
	public async set(serverId: string, image: Blob): Promise<Archon.Icons.v1.ServerIcon> {
		return this.client.request<Archon.Icons.v1.ServerIcon>(
			`/servers/${encodeURIComponent(serverId)}/icon`,
			{
				api: 'archon',
				version: 1,
				method: 'PUT',
				headers: { 'Content-Type': 'application/octet-stream' },
				body: image,
			},
		)
	}

	/**
	 * Upload an icon created with the icon editor, along with its config.
	 * PUT /v1/servers/:id/icon/generated
	 */
	public async setGenerated(
		serverId: string,
		image: Blob,
		config: IconConfig,
	): Promise<Archon.Icons.v1.ServerIcon> {
		const formData = new FormData()
		formData.append('image', image)
		formData.append('config', JSON.stringify(config))

		return this.client.request<Archon.Icons.v1.ServerIcon>(
			`/servers/${encodeURIComponent(serverId)}/icon/generated`,
			{
				api: 'archon',
				version: 1,
				method: 'PUT',
				headers: { 'Content-Type': '' },
				body: formData,
			},
		)
	}

	/**
	 * Delete a server's custom icon.
	 * DELETE /v1/servers/:id/icon
	 */
	public async delete(serverId: string): Promise<void> {
		return this.client.request<void>(`/servers/${encodeURIComponent(serverId)}/icon`, {
			api: 'archon',
			version: 1,
			method: 'DELETE',
		})
	}
}
