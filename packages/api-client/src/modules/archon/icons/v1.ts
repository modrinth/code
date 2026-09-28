import { AbstractModule } from '../../../core/abstract-module'

export class ArchonIconsV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_icons_v1'
	}

	public async get(serverId: string): Promise<Blob> {
		return this.client.request<Blob>(`/servers/${encodeURIComponent(serverId)}/icon`, {
			api: 'archon',
			version: 1,
			method: 'GET',
			responseType: 'blob',
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
