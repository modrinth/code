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
