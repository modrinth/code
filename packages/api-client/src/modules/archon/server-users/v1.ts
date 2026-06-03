import { AbstractModule } from '../../../core/abstract-module'
import type { Archon } from '../types'

export class ArchonServerUsersV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'archon_server_users_v1'
	}

	/**
	 * Get list of users with access to a server
	 * GET /v1/servers/:server_id/users
	 */
	public async list(serverId: string): Promise<Archon.ServerUsers.v1.ServerUser[]> {
		return this.client.request<Archon.ServerUsers.v1.ServerUser[]>(`/servers/${serverId}/users`, {
			api: 'archon',
			version: 1,
			method: 'GET',
		})
	}

	/**
	 * Add a user to a server
	 * POST /v1/servers/:server_id/users
	 */
	public async add(
		serverId: string,
		user: Archon.ServerUsers.v1.AddServerUserRequest,
	): Promise<void> {
		await this.client.request(`/servers/${serverId}/users`, {
			api: 'archon',
			version: 1,
			method: 'POST',
			body: user,
		})
	}

	/**
	 * Re-send an invite to a pending server user.
	 * POST /v1/servers/:server_id/users/:user_id/reinvite
	 */
	public async reinvite(
		serverId: string,
		userId: string,
	): Promise<Archon.ServerUsers.v1.ReinviteResponse> {
		return this.client.request<Archon.ServerUsers.v1.ReinviteResponse>(
			`/servers/${serverId}/users/${userId}/reinvite`,
			{
				api: 'archon',
				version: 1,
				method: 'POST',
			},
		)
	}

	/**
	 * Remove a user from a server
	 * DELETE /v1/servers/:server_id/users/:user_id
	 */
	public async delete(serverId: string, userId: string): Promise<void> {
		await this.client.request(`/servers/${serverId}/users/${userId}`, {
			api: 'archon',
			version: 1,
			method: 'DELETE',
		})
	}

	/**
	 * Update a user's server role
	 * PATCH /v1/servers/:server_id/users/:user_id
	 */
	public async update(
		serverId: string,
		userId: string,
		role: Archon.ServerUsers.v1.AssignableServerUserRole,
	): Promise<void> {
		await this.client.request(`/servers/${serverId}/users/${userId}`, {
			api: 'archon',
			version: 1,
			method: 'PATCH',
			body: JSON.stringify(role),
		})
	}
}
