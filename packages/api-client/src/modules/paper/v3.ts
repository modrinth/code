import { AbstractModule } from '../../core/abstract-module'
import type { Paper } from './types'

export type { Paper } from './types'

const PAPER_BASE_URL = 'https://fill.papermc.io'

export class PaperVersionsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'paper_versions_v3'
	}

	/**
	 * Get the Paper project info including all supported Minecraft versions.
	 */
	public async getProject(): Promise<Paper.Versions.v3.Project> {
		return this.client.request<Paper.Versions.v3.Project>('/projects/paper', {
			api: PAPER_BASE_URL,
			version: 'v3',
			method: 'GET',
			skipAuth: true,
		})
	}

	/**
	 * Get available Paper builds for a Minecraft version (includes channel per build).
	 *
	 * Fill (`fill.papermc.io`) returns a JSON array of builds at this path — not a `{ builds }`
	 * wrapper like some other Paper API shapes — so we normalize to `VersionBuilds`.
	 *
	 * @param mcVersion - Minecraft version (e.g. "1.21.4")
	 */
	public async getBuilds(mcVersion: string): Promise<Paper.Versions.v3.VersionBuilds> {
		const builds = await this.client.request<Paper.Versions.v3.Build[]>(
			`/projects/paper/versions/${mcVersion}/builds`,
			{ api: PAPER_BASE_URL, version: 'v3', method: 'GET', skipAuth: true },
		)
		return { builds }
	}
}
