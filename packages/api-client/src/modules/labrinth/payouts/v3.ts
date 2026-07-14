import { AbstractModule } from '../../../core/abstract-module.js'
import type { Labrinth } from '../types'

export class LabrinthPayoutsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_payouts_v3'
	}

	/**
	 * Get platform revenue data.
	 *
	 * @param params - Optional start/end date filters
	 * @returns Promise resolving to platform revenue data
	 */
	public async getPlatformRevenue(params?: {
		start?: string
		end?: string
	}): Promise<Labrinth.Payouts.v3.RevenueResponse> {
		return this.client.request<Labrinth.Payouts.v3.RevenueResponse>('/payout/platform_revenue', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
			params: params as Record<string, string>,
		})
	}
}
