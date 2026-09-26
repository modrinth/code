import { AbstractModule } from '../../../core/abstract-module'
import type { Minelog } from '../types'

export class MinelogInsightsV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'minelog_insights_v2'
	}

	/**
	 * Analyses log text without saving it.
	 *
	 * @param content - Raw log text
	 */
	public async analyse(content: string): Promise<Minelog.Insights.v2.InsightsResponse> {
		return this.client.request<Minelog.Insights.v2.InsightsResponse>('/analyse', {
			api: 'https://api.minelog.org',
			version: 2,
			method: 'POST',
			body: { content },
			skipAuth: true,
		})
	}
}
