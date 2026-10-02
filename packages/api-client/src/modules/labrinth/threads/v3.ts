import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthThreadsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_threads_v3'
	}

	/**
	 * Get a thread by ID (v3)
	 *
	 * @param id - Thread ID
	 * @returns Promise resolving to the thread data
	 *
	 * @example
	 * ```typescript
	 * const thread = await client.labrinth.threads_v3.getThread('abc123')
	 * console.log(thread.messages)
	 * ```
	 */
	public async getThread(id: string): Promise<Labrinth.Threads.v3.Thread> {
		return this.client.request<Labrinth.Threads.v3.Thread>(`/thread/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	/**
	 * Get multiple threads by IDs (v3)
	 *
	 * @param ids - Array of thread IDs
	 * @returns Promise resolving to an array of threads
	 *
	 * @example
	 * ```typescript
	 * const threads = await client.labrinth.threads_v3.getMultiple(['id1', 'id2'])
	 * ```
	 */
	public async getMultiple(ids: string[]): Promise<Labrinth.Threads.v3.Thread[]> {
		return this.client.request<Labrinth.Threads.v3.Thread[]>(
			`/threads?ids=${encodeURIComponent(JSON.stringify(ids))}`,
			{
				api: 'labrinth',
				version: 3,
				method: 'GET',
			},
		)
	}

	/**
	 * Send a message to a thread (v3)
	 *
	 * @param id - Thread ID
	 * @param message - Message body to send
	 * @returns Promise resolving when message is sent
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.threads_v3.sendMessage('abc123', {
	 *   body: { type: 'text', body: 'Hello!' }
	 * })
	 * ```
	 */
	public async sendMessage(
		id: string,
		message: Labrinth.Threads.v3.SendMessageRequest,
	): Promise<void> {
		return this.client.request(`/thread/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: message,
		})
	}

	/** Create moderation issues in a thread (requires THREAD_WRITE scope). */
	public async createIssues(id: string, data: Labrinth.Threads.v3.NewThreadIssues): Promise<void> {
		return this.client.request<void>(`/thread/${id}/issue`, {
			api: 'labrinth',
			version: 3,
			method: 'PUT',
			body: data,
		})
	}

	/** Update a thread issue's content, facets, or addressed and verified state. */
	public async editIssue(id: string, data: Labrinth.Threads.v3.EditThreadIssue): Promise<void> {
		return this.client.request<void>(`/thread/issue/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'PATCH',
			body: data,
		})
	}

	/** Mark a thread issue as addressed by the project member. */
	public async user_addressed(issueId: string): Promise<void> {
		return this.editIssue(issueId, { user_addressed: true })
	}

	/** Mark a thread issue as verified by a moderator. */
	public async moderator_verified(issueId: string): Promise<void> {
		return this.editIssue(issueId, { moderator_verified: true })
	}

	/**
	 * Delete a message from a thread (v3)
	 *
	 * @param messageId - Message ID
	 * @returns Promise resolving when message is deleted
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.threads_v3.deleteMessage('msg123')
	 * ```
	 */
	public async deleteMessage(messageId: string): Promise<void> {
		return this.client.request(`/message/${messageId}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}
}
