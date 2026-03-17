import { AbstractModule } from '../../../core/abstract-module.js'
import type { Labrinth } from '../types'

export class LabrinthLimitsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_limits_v3'
	}

	/**
	 * Get project creation limits for the authenticated user.
	 */
	public async getProjectLimits(): Promise<Labrinth.Limits.v3.UserLimits> {
		return this.client.request<Labrinth.Limits.v3.UserLimits>('/limits/projects', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	/**
	 * Get organization creation limits for the authenticated user.
	 */
	public async getOrganizationLimits(): Promise<Labrinth.Limits.v3.UserLimits> {
		return this.client.request<Labrinth.Limits.v3.UserLimits>('/limits/organizations', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	/**
	 * Get collection creation limits for the authenticated user.
	 */
	public async getCollectionLimits(): Promise<Labrinth.Limits.v3.UserLimits> {
		return this.client.request<Labrinth.Limits.v3.UserLimits>('/limits/collections', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}
}
