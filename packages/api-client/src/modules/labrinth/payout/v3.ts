import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthPayoutV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_payout_v3'
	}

	/**
	 * Get the authenticated user's payout balance
	 *
	 * @returns Promise resolving to the user's payout balance
	 */
	public async getBalance(): Promise<Labrinth.Payout.v3.PayoutBalance> {
		return this.client.request<Labrinth.Payout.v3.PayoutBalance>('/payout/balance', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}
}
