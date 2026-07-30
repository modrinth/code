import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthAnalyticsInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_analytics_internal'
	}

	/**
	 * Create an analytics event.
	 * POST /_internal/analytics-event
	 */
	public async createEvent(
		data: Labrinth.Analytics.Internal.AnalyticsEventUpsert,
	): Promise<Labrinth.Analytics.v3.AnalyticsEvent> {
		return this.client.request<Labrinth.Analytics.v3.AnalyticsEvent>('/analytics-event', {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Edit an analytics event.
	 * PATCH /_internal/analytics-event/{id}
	 */
	public async editEvent(
		id: Labrinth.Analytics.v3.AnalyticsEventId,
		data: Labrinth.Analytics.Internal.AnalyticsEventUpsert,
	): Promise<Labrinth.Analytics.v3.AnalyticsEvent> {
		return this.client.request<Labrinth.Analytics.v3.AnalyticsEvent>(`/analytics-event/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'PATCH',
			body: data,
		})
	}

	/**
	 * Delete an analytics event.
	 * DELETE /_internal/analytics-event/{id}
	 */
	public async deleteEvent(id: Labrinth.Analytics.v3.AnalyticsEventId): Promise<void> {
		return this.client.request<void>(`/analytics-event/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'DELETE',
		})
	}
}
