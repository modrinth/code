import { $fetch } from 'ofetch'

import { AbstractModule } from '../../core/abstract-module'
import type { Purpur } from './types'

export type { Purpur } from './types'

const BASE_URL = 'https://api.purpurmc.org/v2'

export class PurpurVersionsV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'purpur_versions_v2'
	}

	/**
	 * Get available Purpur builds for a Minecraft version.
	 *
	 * @param mcVersion - Minecraft version (e.g. "1.21.4")
	 */
	public async getBuilds(mcVersion: string): Promise<Purpur.Versions.v2.VersionBuilds> {
		return $fetch<Purpur.Versions.v2.VersionBuilds>(`${BASE_URL}/purpur/${mcVersion}`)
	}
}
