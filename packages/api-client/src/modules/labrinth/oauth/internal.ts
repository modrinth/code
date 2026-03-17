import { AbstractModule } from '../../../core/abstract-module'
import type { UploadHandle } from '../../../types/upload'
import type { Labrinth } from '../types'

export class LabrinthOAuthInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_oauth_internal'
	}

	/**
	 * Get a user's OAuth applications
	 *
	 * @param userId - The user's ID
	 * @returns Promise resolving to an array of the user's OAuth clients
	 */
	public async getUserApps(userId: string): Promise<Labrinth.OAuth.Internal.OAuthClient[]> {
		return this.client.request<Labrinth.OAuth.Internal.OAuthClient[]>(
			`/user/${userId}/oauth_apps`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Get a single OAuth application by ID
	 *
	 * @param id - The OAuth client ID
	 * @returns Promise resolving to the OAuth client
	 */
	public async getApp(id: string): Promise<Labrinth.OAuth.Internal.OAuthClient> {
		return this.client.request<Labrinth.OAuth.Internal.OAuthClient>(`/oauth/app/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
		})
	}

	/**
	 * Get multiple OAuth applications by their IDs
	 *
	 * @param ids - Array of OAuth client IDs
	 * @returns Promise resolving to an array of OAuth clients
	 */
	public async getApps(ids: string[]): Promise<Labrinth.OAuth.Internal.OAuthClient[]> {
		return this.client.request<Labrinth.OAuth.Internal.OAuthClient[]>(
			`/oauth/apps?ids=${encodeURIComponent(JSON.stringify(ids))}`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	/**
	 * Create a new OAuth application
	 *
	 * @param data - The OAuth app creation data
	 * @returns Promise resolving to the created OAuth client with its client secret
	 */
	public async createApp(
		data: Labrinth.OAuth.Internal.CreateOAuthAppRequest,
	): Promise<Labrinth.OAuth.Internal.OAuthClientCreationResult> {
		return this.client.request<Labrinth.OAuth.Internal.OAuthClientCreationResult>(
			`/oauth/app`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'POST',
				body: data,
			},
		)
	}

	/**
	 * Edit an existing OAuth application
	 *
	 * @param id - The OAuth client ID
	 * @param data - The fields to update
	 */
	public async editApp(
		id: string,
		data: Labrinth.OAuth.Internal.EditOAuthAppRequest,
	): Promise<void> {
		return this.client.request(`/oauth/app/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'PATCH',
			body: data,
		})
	}

	/**
	 * Delete an OAuth application
	 *
	 * @param id - The OAuth client ID
	 */
	public async deleteApp(id: string): Promise<void> {
		return this.client.request(`/oauth/app/${id}`, {
			api: 'labrinth',
			version: 'internal',
			method: 'DELETE',
		})
	}

	/**
	 * Update the icon for an OAuth application
	 *
	 * @param id - The OAuth client ID
	 * @param file - The icon file
	 * @param ext - The file extension (e.g. 'png', 'jpeg')
	 * @returns UploadHandle for progress tracking and cancellation
	 */
	public uploadAppIcon(id: string, file: File | Blob, ext: string): UploadHandle<void> {
		return this.client.upload<void>(`/oauth/app/${id}/icon`, {
			api: 'labrinth',
			version: 'internal',
			file,
			params: { ext },
		})
	}

	/**
	 * Get the current user's OAuth authorizations
	 *
	 * @returns Promise resolving to an array of OAuth client authorizations
	 */
	public async getAuthorizations(): Promise<Labrinth.OAuth.Internal.OAuthClientAuthorization[]> {
		return this.client.request<Labrinth.OAuth.Internal.OAuthClientAuthorization[]>(
			`/oauth/authorizations`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
			},
		)
	}

	/**
	 * Revoke an OAuth authorization for a client
	 *
	 * @param clientId - The OAuth client ID to revoke
	 */
	public async revokeAuthorization(clientId: string): Promise<void> {
		return this.client.request(`/oauth/authorizations`, {
			api: 'labrinth',
			version: 'internal',
			method: 'DELETE',
			params: { client_id: clientId },
		})
	}

	/**
	 * Initialize an OAuth authorization flow
	 *
	 * Returns either an OAuthClientAccessRequest (if user needs to approve)
	 * or a redirect URL string (if already authorized).
	 *
	 * @param params - The OAuth query parameters
	 * @returns Promise resolving to an access request object or redirect URL string
	 */
	public async authorize(params: {
		client_id: string
		redirect_uri: string
		scope: string
		state?: string
	}): Promise<Labrinth.OAuth.Internal.OAuthClientAccessRequest | string> {
		return this.client.request<Labrinth.OAuth.Internal.OAuthClientAccessRequest | string>(
			`/oauth/authorize`,
			{
				api: 'labrinth',
				version: 'internal',
				method: 'GET',
				params: params as Record<string, string>,
			},
		)
	}

	/**
	 * Accept an OAuth authorization request
	 *
	 * @param data - The flow ID to accept
	 * @returns Promise resolving to a redirect URL string
	 */
	public async accept(data: Labrinth.OAuth.Internal.AcceptRejectRequest): Promise<string> {
		return this.client.request<string>(`/oauth/accept`, {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Reject an OAuth authorization request
	 *
	 * @param data - The flow ID to reject
	 * @returns Promise resolving to a redirect URL string
	 */
	public async reject(data: Labrinth.OAuth.Internal.AcceptRejectRequest): Promise<string> {
		return this.client.request<string>(`/oauth/reject`, {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
			body: data,
		})
	}
}
