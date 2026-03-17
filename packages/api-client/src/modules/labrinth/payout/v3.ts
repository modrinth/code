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

	/**
	 * Get the authenticated user's transaction history (withdrawals and payouts)
	 *
	 * @returns Promise resolving to an array of transaction items
	 */
	public async getHistory(): Promise<Labrinth.Payout.v3.TransactionItem[]> {
		return this.client.request<Labrinth.Payout.v3.TransactionItem[]>('/payout/history', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	/**
	 * Get available payout methods, optionally filtered by country
	 *
	 * @param country - Optional ISO country code to filter methods by supported countries
	 * @returns Promise resolving to an array of payout methods
	 */
	public async getMethods(country?: string): Promise<Labrinth.Payout.v3.PayoutMethod[]> {
		return this.client.request<Labrinth.Payout.v3.PayoutMethod[]>('/payout/methods', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
			params: country ? { country } : undefined,
		})
	}

	/**
	 * Cancel a pending payout
	 *
	 * @param id - The payout ID to cancel
	 */
	public async cancel(id: string): Promise<void> {
		return this.client.request<void>(`/payout/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}
}
