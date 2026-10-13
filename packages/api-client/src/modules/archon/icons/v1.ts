import { AbstractModule } from '../../../core/abstract-module'

export class ArchonIconsV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_icons_v1'
	}

	/**
	 * Get a server's custom icon.
	 * GET /v1/servers/:id/icon?original=true
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

	public async set(serverId: string, image: Blob): Promise<void> {
		return this.client.request<void>(`/servers/${encodeURIComponent(serverId)}/icon`, {
			api: 'archon',
			version: 1,
			method: 'PUT',
			headers: { 'Content-Type': 'application/octet-stream' },
			body: image,
		})
	}

	public async delete(serverId: string): Promise<void> {
		return this.client.request<void>(`/servers/${encodeURIComponent(serverId)}/icon`, {
			api: 'archon',
			version: 1,
			method: 'DELETE',
		})
	}
}
