import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthTagsV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_tags_v2'
	}

	/** Get SPDX licenses available from the backend. */
	public async getLicenses(): Promise<Labrinth.Tags.v2.License[]> {
		return this.client.request<Labrinth.Tags.v2.License[]>('/tag/license', {
			api: 'labrinth',
			version: 2,
			method: 'GET',
		})
	}

	/**
	 * Get license text by SPDX identifier
	 *
	 * @param licenseId - SPDX license identifier (e.g., 'MIT', 'Apache-2.0')
	 * @returns Promise resolving to the license title and body text
	 *
	 * @example
	 * ```typescript
	 * const license = await client.labrinth.tags_v2.getLicenseText('MIT')
	 * console.log(license.title) // "MIT License"
	 * console.log(license.body)  // full license text
	 * ```
	 */
	public async getLicenseText(licenseId: string): Promise<Labrinth.Tags.v2.LicenseText> {
		return this.client.request<Labrinth.Tags.v2.LicenseText>(`/tag/license/${licenseId}`, {
			api: 'labrinth',
			version: 2,
			method: 'GET',
		})
	}
}
