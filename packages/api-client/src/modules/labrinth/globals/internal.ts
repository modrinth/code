import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthGlobalsInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_globals_internal'
	}

	/**
	 * Get configured global non-secret variables for this backend instance
	 *
	 * @returns Promise resolving to the global configuration
	 */
	public async get(): Promise<Labrinth.Globals.Internal.Globals> {
		return this.client.request<Labrinth.Globals.Internal.Globals>(`/globals`, {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
			skipAuth: true,
		})
	}
}
