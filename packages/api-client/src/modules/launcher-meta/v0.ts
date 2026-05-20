import { AbstractModule } from '../../core/abstract-module'
import type { LauncherMeta } from './types'

export type { LauncherMeta } from './types'

const LAUNCHER_META_BASE_URL = 'https://launcher-meta.modrinth.com'

export class LauncherMetaManifestV0Module extends AbstractModule {
	public getModuleID(): string {
		return 'launchermeta_manifest_v0'
	}

	/**
	 * Get the loader manifest for a given loader platform.
	 *
	 * launcher-meta refuses CORS preflights that ask for the `Content-Type`
	 * header (returns 403), so we strip the default `Content-Type: application/json`
	 * the abstract client sets — these are body-less GETs and don't need it.
	 * Without this the browser preflight is rejected and the GET never fires.
	 *
	 * @param loader - Loader platform (fabric, forge, quilt, neo)
	 */
	public async getManifest(loader: string): Promise<LauncherMeta.Manifest.v0.Manifest> {
		return this.client.request<LauncherMeta.Manifest.v0.Manifest>('/manifest.json', {
			api: LAUNCHER_META_BASE_URL,
			version: `${loader}/v0`,
			method: 'GET',
			skipAuth: true,
			headers: { 'Content-Type': '' },
		})
	}
}
