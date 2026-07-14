import { AbstractModule } from '../../../core/abstract-module'
import type { Mclogs } from '../types'

export class MclogsInsightsV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'mclogs_insights_v1'
	}

	public async analyse(content: string): Promise<Mclogs.Insights.v1.InsightsResponse> {
		return this.client.request<Mclogs.Insights.v1.InsightsResponse>('/analyse', {
			api: 'https://api.mclo.gs',
			version: '1',
			method: 'POST',
			body: new URLSearchParams({ content }),
			headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
			skipAuth: true,
		})
	}
}
