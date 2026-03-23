import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthAuthInternalModule extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_auth_internal'
	}

	/**
	 * Check if the user is subscribed to the newsletter
	 *
	 * @returns Promise resolving to the subscription status
	 */
	public async getNewsletterStatus(): Promise<Labrinth.Auth.Internal.SubscriptionStatus> {
		return this.client.request<Labrinth.Auth.Internal.SubscriptionStatus>('/auth/email/subscribe', {
			api: 'labrinth',
			version: 'internal',
			method: 'GET',
		})
	}

	/**
	 * Subscribe to the newsletter
	 */
	public async subscribeNewsletter(): Promise<void> {
		return this.client.request('/auth/email/subscribe', {
			api: 'labrinth',
			version: 'internal',
			method: 'POST',
		})
	}
}
