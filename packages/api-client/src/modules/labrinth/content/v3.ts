import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthContentV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_content_v3'
	}

	public async resolve(
		request: Labrinth.Content.v3.ResolveContentRequest,
	): Promise<Labrinth.Content.v3.ResolveContentPlan> {
		return this.client.request<Labrinth.Content.v3.ResolveContentPlan>('/content/resolve', {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: request,
		})
	}
}
