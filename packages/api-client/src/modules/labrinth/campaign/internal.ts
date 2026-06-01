import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthCampaignInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_campaign_internal'
	}

	/**
	 * Get Pride 2026 campaign fundraising progress.
	 * GET /_internal/campaign/pride-26
	 */
	public async getPride26(): Promise<Labrinth.Campaign.Internal.CampaignInfo> {
		return this.client.request<Labrinth.Campaign.Internal.CampaignInfo>('/campaign/pride-26', {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
			skipAuth: true,
		})
	}
}
