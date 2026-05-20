import { AbstractModule } from '../../../core/abstract-module'
import type { Mclogs } from '../types'

export class MclogsLogsV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'mclogs_logs_v1'
	}

	public async create(content: string): Promise<Mclogs.Logs.v1.CreateResponse> {
		return this.client.request<Mclogs.Logs.v1.CreateResponse>('/log', {
			api: 'https://api.mclo.gs',
			version: '1',
			method: 'POST',
			body: new URLSearchParams({ content }),
			headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
			skipAuth: true,
		})
	}
}
