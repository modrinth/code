import { $fetch } from 'ofetch'

import { AbstractModule } from '../../core/abstract-module'
import type { Paper } from './types'

export type { Paper } from './types'

const BASE_URL = 'https://fill.papermc.io/v3'

export class PaperVersionsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'paper_versions_v3'
	}

	/**
	 * Get available Paper builds for a Minecraft version.
	 *
	 * @param mcVersion - Minecraft version (e.g. "1.21.4")
	 */
	public async getBuilds(mcVersion: string): Promise<Paper.Versions.v3.VersionBuilds> {
		return $fetch<Paper.Versions.v3.VersionBuilds>(
			`${BASE_URL}/projects/paper/versions/${mcVersion}`,
		)
	}
}
