import { AbstractModule } from '../../core/abstract-module'
import type { Purpur } from './types'

export type { Purpur } from './types'

const PURPUR_BASE_URL = 'https://api.purpurmc.org'

export class PurpurVersionsV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'purpur_versions_v2'
	}

	/**
	 * Get the Purpur project info including all supported Minecraft versions.
	 */
	public async getProject(): Promise<Purpur.Versions.v2.Project> {
		return this.client.request<Purpur.Versions.v2.Project>('/purpur', {
			api: PURPUR_BASE_URL,
			version: 'v2',
			method: 'GET',
			skipAuth: true,
		})
	}

	/**
	 * Get available Purpur builds for a Minecraft version.
	 *
	 * @param mcVersion - Minecraft version (e.g. "1.21.4")
	 */
	public async getBuilds(mcVersion: string): Promise<Purpur.Versions.v2.VersionBuilds> {
		return this.client.request<Purpur.Versions.v2.VersionBuilds>(`/purpur/${mcVersion}`, {
			api: PURPUR_BASE_URL,
			version: 'v2',
			method: 'GET',
			skipAuth: true,
		})
	}
}
