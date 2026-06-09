import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

const mockHistory: Labrinth.Payouts.Internal.HistoryItem[] = [
	{
		payouts_date: '2025-04',
		days: createMockRevenueDays('2025-04', 24_150),
		status: 'open',
		fees_deducted_usd: 1_440,
		variance_adjustment_usd: -2_650,
		net_estimated_revenue_usd: 20_060,
		creator_net_estimated_revenue_usd: 15_045,
		modrinth_net_estimated_revenue_usd: 5_015,
		started_at: null,
		started_by: null,
		detailed_external_adjustments: null,
	},
	{
		payouts_date: '2025-03',
		days: createMockRevenueDays('2025-03', 48_200),
		status: 'pending',
		fees_deducted_usd: 1_440,
		variance_adjustment_usd: -2_650,
		net_estimated_revenue_usd: 44_110,
		creator_net_estimated_revenue_usd: 33_083,
		modrinth_net_estimated_revenue_usd: 11_028,
		started_at: null,
		started_by: null,
		detailed_external_adjustments: null,
	},
	{
		payouts_date: '2025-02',
		days: createMockRevenueDays('2025-02', 45_500),
		status: 'pending',
		fees_deducted_usd: 1_312,
		variance_adjustment_usd: -2_500,
		net_estimated_revenue_usd: 41_688,
		creator_net_estimated_revenue_usd: 31_266,
		modrinth_net_estimated_revenue_usd: 10_422,
		started_at: null,
		started_by: null,
		detailed_external_adjustments: null,
	},
	{
		payouts_date: '2025-01',
		days: createMockRevenueDays('2025-01', 42_000),
		status: 'review',
		fees_deducted_usd: 1_200,
		variance_adjustment_usd: -2_100,
		net_estimated_revenue_usd: 38_700,
		creator_net_estimated_revenue_usd: 29_025,
		modrinth_net_estimated_revenue_usd: 9_675,
		started_at: null,
		started_by: null,
		detailed_external_adjustments: null,
	},
	{
		payouts_date: '2024-12',
		days: createMockRevenueDays('2024-12', 51_000),
		status: 'paid',
		fees_deducted_usd: 1_520,
		variance_adjustment_usd: -2_800,
		net_estimated_revenue_usd: 46_680,
		creator_net_estimated_revenue_usd: 35_010,
		modrinth_net_estimated_revenue_usd: 11_670,
		actual_revenue_usd: 48_800,
		total_external_adjustment_usd: 0,
		net_actual_revenue_usd: 48_800,
		creator_net_actual_revenue_usd: 36_600,
		modrinth_net_actual_revenue_usd: 12_200,
		started_at: '2025-03-16T21:10:00.000Z',
		started_by: 'mock-admin-user',
		detailed_external_adjustments: [],
	},
	{
		payouts_date: '2024-11',
		days: createMockRevenueDays('2024-11', 49_500),
		status: 'paid',
		fees_deducted_usd: 1_472,
		variance_adjustment_usd: -2_700,
		net_estimated_revenue_usd: 45_328,
		creator_net_estimated_revenue_usd: 33_996,
		modrinth_net_estimated_revenue_usd: 11_332,
		actual_revenue_usd: 47_500,
		total_external_adjustment_usd: 0,
		net_actual_revenue_usd: 47_500,
		creator_net_actual_revenue_usd: 35_625,
		modrinth_net_actual_revenue_usd: 11_875,
		started_at: '2025-02-14T19:45:00.000Z',
		started_by: 'mock-admin-user',
		detailed_external_adjustments: [],
	},
	{
		payouts_date: '2024-10',
		days: createMockRevenueDays('2024-10', 46_000),
		status: 'paid',
		fees_deducted_usd: 1_360,
		variance_adjustment_usd: -2_500,
		net_estimated_revenue_usd: 42_140,
		creator_net_estimated_revenue_usd: 31_605,
		modrinth_net_estimated_revenue_usd: 10_535,
		actual_revenue_usd: 44_000,
		total_external_adjustment_usd: 0,
		net_actual_revenue_usd: 44_000,
		creator_net_actual_revenue_usd: 33_000,
		modrinth_net_actual_revenue_usd: 11_000,
		started_at: '2025-01-15T20:25:00.000Z',
		started_by: 'mock-admin-user',
		detailed_external_adjustments: [],
	},
	{
		payouts_date: '2024-09',
		days: createMockRevenueDays('2024-09', 44_000),
		status: 'paid',
		fees_deducted_usd: 1_280,
		variance_adjustment_usd: -2_400,
		net_estimated_revenue_usd: 40_320,
		creator_net_estimated_revenue_usd: 30_240,
		modrinth_net_estimated_revenue_usd: 10_080,
		actual_revenue_usd: 42_000,
		total_external_adjustment_usd: 0,
		net_actual_revenue_usd: 42_000,
		creator_net_actual_revenue_usd: 31_500,
		modrinth_net_actual_revenue_usd: 10_500,
		started_at: '2024-12-14T18:20:00.000Z',
		started_by: 'mock-admin-user',
		detailed_external_adjustments: [],
	},
	{
		payouts_date: '2024-08',
		days: createMockRevenueDays('2024-08', 43_500),
		status: 'paid',
		fees_deducted_usd: 1_264,
		variance_adjustment_usd: -2_350,
		net_estimated_revenue_usd: 39_886,
		creator_net_estimated_revenue_usd: 29_915,
		modrinth_net_estimated_revenue_usd: 9_972,
		actual_revenue_usd: 39_800,
		total_external_adjustment_usd: 0,
		net_actual_revenue_usd: 39_800,
		creator_net_actual_revenue_usd: 29_850,
		modrinth_net_actual_revenue_usd: 9_950,
		started_at: '2024-11-15T17:30:00.000Z',
		started_by: 'mock-admin-user',
		detailed_external_adjustments: [],
	},
	{
		payouts_date: '2024-07',
		days: createMockRevenueDays('2024-07', 41_000),
		status: 'paid',
		fees_deducted_usd: 1_200,
		variance_adjustment_usd: -2_200,
		net_estimated_revenue_usd: 37_600,
		creator_net_estimated_revenue_usd: 28_200,
		modrinth_net_estimated_revenue_usd: 9_400,
		actual_revenue_usd: 39_200,
		total_external_adjustment_usd: 0,
		net_actual_revenue_usd: 39_200,
		creator_net_actual_revenue_usd: 29_400,
		modrinth_net_actual_revenue_usd: 9_800,
		started_at: '2024-10-14T17:30:00.000Z',
		started_by: 'mock-admin-user',
		detailed_external_adjustments: [],
	},
	{
		payouts_date: '2024-06',
		days: createMockRevenueDays('2024-06', 39_500),
		status: 'paid',
		fees_deducted_usd: 1_152,
		variance_adjustment_usd: -2_100,
		net_estimated_revenue_usd: 36_248,
		creator_net_estimated_revenue_usd: 27_186,
		modrinth_net_estimated_revenue_usd: 9_062,
		actual_revenue_usd: 37_800,
		total_external_adjustment_usd: 0,
		net_actual_revenue_usd: 37_800,
		creator_net_actual_revenue_usd: 28_350,
		modrinth_net_actual_revenue_usd: 9_450,
		started_at: '2024-09-14T17:30:00.000Z',
		started_by: 'mock-admin-user',
		detailed_external_adjustments: [],
	},
	{
		payouts_date: '2024-05',
		days: createMockRevenueDays('2024-05', 38_000),
		status: 'paid',
		fees_deducted_usd: 1_120,
		variance_adjustment_usd: -2_000,
		net_estimated_revenue_usd: 34_880,
		creator_net_estimated_revenue_usd: 26_160,
		modrinth_net_estimated_revenue_usd: 8_720,
		actual_revenue_usd: 36_200,
		total_external_adjustment_usd: 0,
		net_actual_revenue_usd: 36_200,
		creator_net_actual_revenue_usd: 27_150,
		modrinth_net_actual_revenue_usd: 9_050,
		started_at: '2024-08-15T17:30:00.000Z',
		started_by: 'mock-admin-user',
		detailed_external_adjustments: [],
	},
]

let mockDistribution: Labrinth.Payouts.Internal.DistributionRun | null = null

function createMockRevenueDays(
	payoutsDate: Labrinth.Payouts.Internal.YearMonth,
	totalRevenue: number,
): Labrinth.Payouts.Internal.RevenueDay[] {
	const daysInMonth = getDaysInMonth(payoutsDate)
	const weights = Array.from({ length: daysInMonth }, (_, index) => {
		const weekdayLift = index % 7 === 4 || index % 7 === 5 ? 0.16 : 0
		return 1 + ((index * 7) % 11) / 20 + weekdayLift
	})
	const totalWeight = weights.reduce((total, weight) => total + weight, 0)
	let allocatedRevenue = 0

	return weights.map((weight, index) => {
		const estimatedRevenue =
			index === weights.length - 1
				? totalRevenue - allocatedRevenue
				: Math.round((totalRevenue * weight) / totalWeight)
		allocatedRevenue += estimatedRevenue

		return { estimated_revenue_usd: estimatedRevenue }
	})
}

function getDaysInMonth(payoutsDate: Labrinth.Payouts.Internal.YearMonth): number {
	const [year, month] = payoutsDate.split('-').map(Number)
	return new Date(year, month, 0).getDate()
}

export class LabrinthPayoutsInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_payouts_internal'
	}

	/**
	 * Get creator payout history.
	 * GET /_internal/payouts/history
	 */
	public async getHistory(): Promise<Labrinth.Payouts.Internal.HistoryItem[]> {
		return getMockHistory()

		// return this.client.request<Labrinth.Payouts.Internal.HistoryItem[]>('/payouts/history', {
		// 	api: 'labrinth',
		// 	version: 'internal',
		// 	method: 'GET',
		// })
	}

	/**
	 * Get the active payout distribution run.
	 * GET /_internal/payouts/distribution
	 */
	public async getDistribution(): Promise<Labrinth.Payouts.Internal.DistributionRun | null> {
		return mockDistribution

		// return this.client.request<Labrinth.Payouts.Internal.DistributionRun | null>(
		// 	'/payouts/distribution',
		// 	{
		// 		api: 'labrinth',
		// 		version: 'internal',
		// 		method: 'GET',
		// 	},
		// )
	}

	/**
	 * Start a payout distribution run.
	 * POST /_internal/payouts/distribution/start
	 */
	public async startDistribution(
		data: Labrinth.Payouts.Internal.StartDistributionRequest,
	): Promise<Labrinth.Payouts.Internal.DistributionRun> {
		const startedAt = new Date()
		mockDistribution = {
			payouts_date: data.payouts_date,
			amount_received: data.amount_received,
			adjustments: data.adjustments,
			started_at: startedAt.toISOString(),
			started_by: 'mock-admin-user',
			distributes_at: new Date(startedAt.getTime() + 2 * 60 * 1000).toISOString(),
		}

		return mockDistribution

		// return this.client.request<Labrinth.Payouts.Internal.DistributionRun>(
		// 	'/payouts/distribution/start',
		// 	{
		// 		api: 'labrinth',
		// 		version: 'internal',
		// 		method: 'POST',
		// 		body: data,
		// 	},
		// )
	}

	/**
	 * Cancel the active payout distribution run.
	 * POST /_internal/payouts/distribution/cancel
	 */
	public async cancelDistribution(): Promise<void> {
		mockDistribution = null

		// return this.client.request<void>('/payouts/distribution/cancel', {
		// 	api: 'labrinth',
		// 	version: 'internal',
		// 	method: 'POST',
		// })
	}
}

function getMockHistory(): Labrinth.Payouts.Internal.HistoryItem[] {
	if (!mockDistribution) {
		return mockHistory
	}

	const activeDistribution = mockDistribution
	return mockHistory.map((payout) =>
		payout.payouts_date === activeDistribution.payouts_date
			? {
					...payout,
					started_at: activeDistribution.started_at,
					started_by: activeDistribution.started_by,
					detailed_external_adjustments: activeDistribution.adjustments.map((adjustment) => ({
						description: adjustment.description,
						amount_usd: adjustment.amount,
					})),
				}
			: payout,
	)
}
