import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthNotificationsV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_notifications_v2'
	}

	/**
	 * Get all notifications for a user
	 *
	 * @param userId - The user's ID
	 * @returns Promise resolving to the user's notifications
	 *
	 * @example
	 * ```typescript
	 * const notifications = await client.labrinth.notifications_v2.getUserNotifications('user123')
	 * ```
	 */
	public async getUserNotifications(
		userId: string,
	): Promise<Labrinth.Notifications.v2.Notification[]> {
		return this.client.request<Labrinth.Notifications.v2.Notification[]>(
			`/user/${userId}/notifications`,
			{
				api: 'labrinth',
				version: 2,
				method: 'GET',
			},
		)
	}

	/**
	 * Get multiple notifications by their IDs
	 *
	 * @param ids - Array of notification IDs
	 * @returns Promise resolving to an array of notifications
	 *
	 * @example
	 * ```typescript
	 * const notifications = await client.labrinth.notifications_v2.getMultiple(['id1', 'id2'])
	 * ```
	 */
	public async getMultiple(ids: string[]): Promise<Labrinth.Notifications.v2.Notification[]> {
		return this.client.request<Labrinth.Notifications.v2.Notification[]>(
			`/notifications?ids=${encodeURIComponent(JSON.stringify(ids))}`,
			{
				api: 'labrinth',
				version: 2,
				method: 'GET',
			},
		)
	}

	/**
	 * Mark a single notification as read
	 *
	 * @param id - Notification ID
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.notifications_v2.markAsRead('notif123')
	 * ```
	 */
	public async markAsRead(id: string): Promise<void> {
		return this.client.request(`/notification/${id}`, {
			api: 'labrinth',
			version: 2,
			method: 'PATCH',
		})
	}

	/**
	 * Mark multiple notifications as read
	 *
	 * @param ids - Array of notification IDs to mark as read
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.notifications_v2.markMultipleAsRead(['id1', 'id2'])
	 * ```
	 */
	public async markMultipleAsRead(ids: string[]): Promise<void> {
		return this.client.request(`/notifications`, {
			api: 'labrinth',
			version: 2,
			method: 'PATCH',
			params: { ids: JSON.stringify([...new Set(ids)]) },
		})
	}

	/**
	 * Delete a single notification
	 *
	 * @param id - Notification ID
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.notifications_v2.delete('notif123')
	 * ```
	 */
	public async delete(id: string): Promise<void> {
		return this.client.request(`/notification/${id}`, {
			api: 'labrinth',
			version: 2,
			method: 'DELETE',
		})
	}

	/**
	 * Delete multiple notifications
	 *
	 * @param ids - Array of notification IDs to delete
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.notifications_v2.deleteMultiple(['id1', 'id2'])
	 * ```
	 */
	public async deleteMultiple(ids: string[]): Promise<void> {
		return this.client.request(`/notifications`, {
			api: 'labrinth',
			version: 2,
			method: 'DELETE',
			params: { ids: JSON.stringify([...new Set(ids)]) },
		})
	}
}
