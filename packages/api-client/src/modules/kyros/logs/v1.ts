import { AbstractModule } from '../../../core/abstract-module'

export class KyrosLogsV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'kyros_logs_v1'
	}

	/** POST /v1/logs/clear — clear the live logs buffer for the current server */
	public async clear(): Promise<void> {
		return this.client.request<void>('/logs/clear', {
			api: '',
			version: 'v1',
			method: 'POST',
			useNodeAuth: true,
		})
	}
}
