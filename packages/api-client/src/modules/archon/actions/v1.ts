import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonActionsV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_actions_v1'
	}

	/**
	 * Get server action log entries.
	 * GET /v1/servers/:server_id/action-log
	 */
	public async list(
		serverId: string,
		options: Archon.Actions.v1.ListActionLogOptions = {},
	): Promise<Archon.Actions.v1.ActionLogResponse> {
		const params: Record<string, string | number> = {}
		if (options.filter) params.filter = JSON.stringify(options.filter)
		if (options.limit !== undefined) params.limit = options.limit
		if (options.offset !== undefined) params.offset = options.offset
		if (options.order !== undefined) params.order = options.order

		return this.client.request<Archon.Actions.v1.ActionLogResponse>(
			`/servers/${serverId}/action-log`,
			{
				api: 'archon',
				version: 1,
				method: 'GET',
				params: Object.keys(params).length > 0 ? params : undefined,
			},
		)
	}
}
