import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthSessionsV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_sessions_v2'
	}

	/**
	 * List all sessions for the authenticated user
	 *
	 * @returns Promise resolving to an array of sessions
	 */
	public async list(): Promise<Labrinth.Sessions.v2.Session[]> {
		return this.client.request<Labrinth.Sessions.v2.Session[]>('/session/list', {
			api: 'labrinth',
			version: 2,
			method: 'GET',
		})
	}

	/**
	 * Delete (revoke) a session
	 *
	 * @param id - The session ID
	 */
	public async delete(id: string): Promise<void> {
		return this.client.request(`/session/${id}`, {
			api: 'labrinth',
			version: 2,
			method: 'DELETE',
		})
	}
}
