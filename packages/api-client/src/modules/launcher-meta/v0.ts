import { $fetch } from 'ofetch'

import { AbstractModule } from '../../core/abstract-module'
import type { LauncherMeta } from './types'

export type { LauncherMeta } from './types'

const BASE_URL = 'https://launcher-meta.modrinth.com'

export class LauncherMetaManifestV0Module extends AbstractModule {
	public getModuleID(): string {
		return 'launchermeta_manifest_v0'
	}

	/**
	 * Get the loader manifest for a given loader platform.
	 *
	 * @param loader - Loader platform (fabric, forge, quilt, neo)
	 */
	public async getManifest(loader: string): Promise<LauncherMeta.Manifest.v0.Manifest> {
		return $fetch<LauncherMeta.Manifest.v0.Manifest>(
			`${BASE_URL}/${loader}/v0/manifest.json`,
		)
	}
}
