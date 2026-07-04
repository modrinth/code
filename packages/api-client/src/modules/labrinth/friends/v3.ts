import { AbstractModule } from '../../../core/abstract-module.js'
import type { Labrinth } from '../types.js'

export class LabrinthFriendsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_friends_v3'
	}

	/**
	 * Get friends and pending friend requests for the authenticated user
	 *
	 * @returns Promise resolving to friend relationships
	 */
	public async list(): Promise<Labrinth.Friends.v3.UserFriend[]> {
		return this.client.request<Labrinth.Friends.v3.UserFriend[]>('/friends', {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	/**
	 * Send or accept a friend request
	 *
	 * @param idOrUsername - The target user's ID or username
	 */
	public async add(idOrUsername: string): Promise<void> {
		return this.client.request(`/friend/${encodeURIComponent(idOrUsername)}`, {
			api: 'labrinth',
			version: 3,
			method: 'POST',
		})
	}

	/**
	 * Remove a friend or pending friend request
	 *
	 * @param idOrUsername - The target user's ID or username
	 */
	public async remove(idOrUsername: string): Promise<void> {
		return this.client.request(`/friend/${encodeURIComponent(idOrUsername)}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}
}
