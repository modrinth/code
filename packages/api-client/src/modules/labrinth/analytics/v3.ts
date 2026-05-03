import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthAnalyticsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_analytics_v3'
	}

	/**
	 * Fetch analytics data for the authenticated user's accessible projects
	 * and affiliate codes.
	 *
	 * @param data - Analytics request body defining time range and requested metrics
	 * @returns Promise resolving to analytics time slices
	 *
	 * @example
	 * ```typescript
	 * const response = await client.labrinth.analytics_v3.fetch({
	 *   time_range: {
	 *     start: '2026-01-01T00:00:00Z',
	 *     end: '2026-02-01T00:00:00Z',
	 *     resolution: { slices: 31 },
	 *   },
	 *   project_ids: ['A1B2C3D4'],
	 *   return_metrics: {
	 *     project_views: { bucket_by: ['project_id'] },
	 *   },
	 * })
	 * ```
	 */
	public async fetch(
		data: Labrinth.Analytics.v3.FetchRequest,
	): Promise<Labrinth.Analytics.v3.FetchResponse> {
		return this.client.request<Labrinth.Analytics.v3.FetchResponse>('/analytics', {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: data,
		})
	}
}
