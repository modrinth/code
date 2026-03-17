import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthAffiliateInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_affiliate_internal'
	}

	/**
	 * Get all affiliate codes for the authenticated user (or all if admin)
	 * GET /_internal/affiliate
	 */
	public async getAll(): Promise<Labrinth.Affiliate.Internal.AffiliateCode[]> {
		return this.client.request<Labrinth.Affiliate.Internal.AffiliateCode[]>('/affiliate', {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
		})
	}

	/**
	 * Create a new affiliate code
	 * PUT /_internal/affiliate
	 */
	public async create(
		data: Labrinth.Affiliate.Internal.CreateRequest,
	): Promise<Labrinth.Affiliate.Internal.AffiliateCode> {
		return this.client.request<Labrinth.Affiliate.Internal.AffiliateCode>('/affiliate', {
			api: 'labrinth',
			version: 'internal',
			method: 'PUT',
			body: data,
		})
	}

	/**
	 * Get a specific affiliate code by ID
	 * GET /_internal/affiliate/{id}
	 */
	public async get(id: string): Promise<Labrinth.Affiliate.Internal.AffiliateCode> {
		return this.client.request<Labrinth.Affiliate.Internal.AffiliateCode>(`/affiliate/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
		})
	}

	/**
	 * Delete an affiliate code
	 * DELETE /_internal/affiliate/{id}
	 */
	public async delete(id: string): Promise<void> {
		return this.client.request<void>(`/affiliate/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'DELETE',
		})
	}

	/**
	 * Update an affiliate code's source name
	 * PATCH /_internal/affiliate/{id}
	 */
	public async patch(id: string, data: Labrinth.Affiliate.Internal.PatchRequest): Promise<void> {
		return this.client.request<void>(`/affiliate/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'PATCH',
			body: data,
		})
	}
}
