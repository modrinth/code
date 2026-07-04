import { AbstractModule } from '../../../core/abstract-module.js'
import type { Labrinth } from '../types.js'

export class LabrinthAnalyticsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_analytics_v3'
	}

	/**
	 * Fetch analytics data for the authenticated user's accessible projects
	 * and affiliate codes.
	 *
	 * @param data - Analytics request body defining time range and requested metrics
	 * @returns Promise resolving to the analytics response, with time slices in `metrics`
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
	 * const timeSlices = response.metrics
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
			timeout: 100 * 1000,
		})
	}

	/**
	 * Fetch available analytics filter facets for the authenticated user's
	 * accessible projects.
	 *
	 * POST /v3/analytics/facets
	 */
	public async fetchFacets(
		data: Labrinth.Analytics.v3.FetchRequest,
	): Promise<Labrinth.Analytics.v3.FacetsResponse> {
		return this.client.request<Labrinth.Analytics.v3.FacetsResponse>('/analytics/facets', {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: data,
			timeout: 100 * 1000,
		})
	}

	/**
	 * Fetch all analytics events.
	 * GET /v3/analytics-event
	 */
	public async getEvents(): Promise<Labrinth.Analytics.v3.AnalyticsEvent[]> {
		return this.client.request<Labrinth.Analytics.v3.AnalyticsEvent[]>('/analytics-event', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	/**
	 * Create an analytics event.
	 * POST /v3/analytics-event
	 */
	public async createEvent(
		data: Labrinth.Analytics.v3.AnalyticsEventUpsert,
	): Promise<Labrinth.Analytics.v3.AnalyticsEvent> {
		return this.client.request<Labrinth.Analytics.v3.AnalyticsEvent>('/analytics-event', {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Edit an analytics event.
	 * PATCH /v3/analytics-event/{id}
	 */
	public async editEvent(
		id: Labrinth.Analytics.v3.AnalyticsEventId,
		data: Labrinth.Analytics.v3.AnalyticsEventUpsert,
	): Promise<Labrinth.Analytics.v3.AnalyticsEvent> {
		return this.client.request<Labrinth.Analytics.v3.AnalyticsEvent>(`/analytics-event/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'PATCH',
			body: data,
		})
	}

	/**
	 * Delete an analytics event.
	 * DELETE /v3/analytics-event/{id}
	 */
	public async deleteEvent(id: Labrinth.Analytics.v3.AnalyticsEventId): Promise<void> {
		return this.client.request<void>(`/analytics-event/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}
}
