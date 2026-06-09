import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthPayoutsInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_payouts_internal'
	}

	/**
	 * Get creator payout history.
	 * GET /_internal/payouts/history
	 */
	public async getHistory(): Promise<Labrinth.Payouts.Internal.HistoryItem[]> {
		return this.client.request<Labrinth.Payouts.Internal.HistoryItem[]>('/payouts/history', {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
		})
	}

	/**
	 * Get the active payout distribution run.
	 * GET /_internal/payouts/distribution
	 */
	public async getDistribution(): Promise<Labrinth.Payouts.Internal.DistributionRun | null> {
		return this.client.request<Labrinth.Payouts.Internal.DistributionRun | null>(
			'/payouts/distribution',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	/**
	 * Start a payout distribution run.
	 * POST /_internal/payouts/distribution/start
	 */
	public async startDistribution(
		data: Labrinth.Payouts.Internal.StartDistributionRequest,
	): Promise<Labrinth.Payouts.Internal.DistributionRun> {
		return this.client.request<Labrinth.Payouts.Internal.DistributionRun>(
			'/payouts/distribution/start',
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
				body: data,
			},
		)
	}

	/**
	 * Cancel the active payout distribution run.
	 * POST /_internal/payouts/distribution/cancel
	 */
	public async cancelDistribution(): Promise<void> {
		return this.client.request<void>('/payouts/distribution/cancel', {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
		})
	}
}
