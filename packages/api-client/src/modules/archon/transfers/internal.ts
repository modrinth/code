import { AbstractModule } from '../../../core/abstract-module.js'
import type { Archon } from '../types.js'

export class ArchonTransfersInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'archon_transfers_internal'
	}

	/**
	 * Schedule transfers for specific servers.
	 * POST /_internal/transfers/schedule/servers
	 */
	public async scheduleServers(
		request: Archon.Transfers.Internal.ScheduleServerTransfersRequest,
	): Promise<Archon.Transfers.Internal.ScheduleTransfersResponse> {
		return this.client.request<Archon.Transfers.Internal.ScheduleTransfersResponse>(
			'/transfers/schedule/servers',
			{
				api: 'archon',
				version: 'internal',
				method: 'POST',
				body: request,
			},
		)
	}

	/**
	 * Schedule transfers for all servers on specific nodes.
	 * POST /_internal/transfers/schedule/nodes
	 */
	public async scheduleNodes(
		request: Archon.Transfers.Internal.ScheduleNodeTransfersRequest,
	): Promise<Archon.Transfers.Internal.ScheduleTransfersResponse> {
		return this.client.request<Archon.Transfers.Internal.ScheduleTransfersResponse>(
			'/transfers/schedule/nodes',
			{
				api: 'archon',
				version: 'internal',
				method: 'POST',
				body: request,
			},
		)
	}

	/**
	 * Get transfer batch history.
	 * GET /_internal/transfers/history
	 */
	public async history(
		options?: Archon.Transfers.Internal.TransferHistoryQuery,
	): Promise<Archon.Transfers.Internal.TransferHistoryResponse> {
		const params: Record<string, number> = {}
		if (options?.page !== undefined) params.page = options.page
		if (options?.page_size !== undefined) params.page_size = options.page_size

		return this.client.request<Archon.Transfers.Internal.TransferHistoryResponse>(
			'/transfers/history',
			{
				api: 'archon',
				version: 'internal',
				method: 'GET',
				params,
			},
		)
	}

	/**
	 * Cancel pending transfer batches.
	 * POST /_internal/transfers/cancel
	 */
	public async cancel(
		request: Archon.Transfers.Internal.CancelTransfersRequest,
	): Promise<Archon.Transfers.Internal.CancelTransfersResponse> {
		return this.client.request<Archon.Transfers.Internal.CancelTransfersResponse>(
			'/transfers/cancel',
			{
				api: 'archon',
				version: 'internal',
				method: 'POST',
				body: request,
			},
		)
	}
}
