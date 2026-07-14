import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthTeamsV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_teams_v2'
	}

	/**
	 * Add a member to a team
	 *
	 * @param teamId - Team ID
	 * @param data - New member data including user_id
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.teams_v2.addMember('team123', { user_id: 'user456' })
	 * ```
	 */
	public async addMember(
		teamId: string,
		data: Labrinth.Teams.v2.AddTeamMemberRequest,
	): Promise<void> {
		return this.client.request(`/team/${teamId}/members`, {
			api: 'labrinth',
			version: 2,
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Edit a team member
	 *
	 * @param teamId - Team ID
	 * @param userId - User ID of the member to edit
	 * @param data - Member update data
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.teams_v2.editMember('team123', 'user456', {
	 *   role: 'Developer',
	 *   permissions: 0b111,
	 * })
	 * ```
	 */
	public async editMember(
		teamId: string,
		userId: string,
		data: Labrinth.Teams.v2.EditTeamMemberRequest,
	): Promise<void> {
		return this.client.request(`/team/${teamId}/members/${userId}`, {
			api: 'labrinth',
			version: 2,
			method: 'PATCH',
			body: data,
		})
	}

	/**
	 * Remove a member from a team
	 *
	 * @param teamId - Team ID
	 * @param userId - User ID of the member to remove
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.teams_v2.removeMember('team123', 'user456')
	 * ```
	 */
	public async removeMember(teamId: string, userId: string): Promise<void> {
		return this.client.request(`/team/${teamId}/members/${userId}`, {
			api: 'labrinth',
			version: 2,
			method: 'DELETE',
		})
	}

	/**
	 * Transfer team ownership to another member
	 *
	 * @param teamId - Team ID
	 * @param data - Transfer data including the new owner's user_id
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.teams_v2.transferOwnership('team123', { user_id: 'user456' })
	 * ```
	 */
	public async transferOwnership(
		teamId: string,
		data: Labrinth.Teams.v2.TransferOwnershipRequest,
	): Promise<void> {
		return this.client.request(`/team/${teamId}/owner`, {
			api: 'labrinth',
			version: 2,
			method: 'PATCH',
			body: data,
		})
	}
}
