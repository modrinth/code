import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthPatsV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_pats_v2'
	}

	/**
	 * Get all personal access tokens for the authenticated user
	 *
	 * @returns Promise resolving to an array of PATs
	 */
	public async list(): Promise<Labrinth.Pats.v2.PersonalAccessToken[]> {
		return this.client.request<Labrinth.Pats.v2.PersonalAccessToken[]>('/pat', {
			api: 'labrinth',
			version: 2,
			method: 'GET',
		})
	}

	/**
	 * Create a new personal access token
	 *
	 * @param data - The PAT creation request data
	 * @returns Promise resolving to the newly created PAT (includes access_token)
	 */
	public async create(
		data: Labrinth.Pats.v2.CreatePatRequest,
	): Promise<Labrinth.Pats.v2.PersonalAccessToken> {
		return this.client.request<Labrinth.Pats.v2.PersonalAccessToken>('/pat', {
			api: 'labrinth',
			version: 2,
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Modify an existing personal access token
	 *
	 * @param id - The PAT ID
	 * @param data - The fields to update
	 */
	public async modify(id: string, data: Labrinth.Pats.v2.ModifyPatRequest): Promise<void> {
		return this.client.request(`/pat/${id}`, {
			api: 'labrinth',
			version: 2,
			method: 'PATCH',
			body: data,
		})
	}

	/**
	 * Delete a personal access token
	 *
	 * @param id - The PAT ID
	 */
	public async delete(id: string): Promise<void> {
		return this.client.request(`/pat/${id}`, {
			api: 'labrinth',
			version: 2,
			method: 'DELETE',
		})
	}
}
