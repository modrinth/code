import { AbstractModule } from '../../../core/abstract-module'
import type { Minelog } from '../types'

export class MinelogLogsV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'minelog_logs_v2'
	}

	/**
	 * Uploads a log to Minelog.
	 *
	 * @param content - Raw log text
	 */
	public async create(content: string): Promise<Minelog.Logs.v2.CreateResponse> {
		return this.client.request<Minelog.Logs.v2.CreateResponse>('/logs', {
			api: 'https://api.minelog.org',
			version: 2,
			method: 'POST',
			body: { content, source: 'Modrinth App' },
			skipAuth: true,
		})
	}
}
