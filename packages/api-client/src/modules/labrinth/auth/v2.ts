import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthAuthV2Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_auth_v2'
	}

	/**
	 * Log in with a password
	 *
	 * Returns a session token on success, or a flow ID if 2FA is required.
	 *
	 * @param data - Login credentials and captcha challenge
	 * @returns Promise resolving to a login response with session or flow
	 */
	public async login(data: Labrinth.Auth.v2.LoginRequest): Promise<Labrinth.Auth.v2.LoginResponse> {
		return this.client.request<Labrinth.Auth.v2.LoginResponse>(`/auth/login`, {
			api: 'labrinth',
			version: 2,
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Complete a 2FA login flow
	 *
	 * @param data - The 2FA code and flow ID
	 * @returns Promise resolving to a session response
	 */
	public async login2FA(
		data: Labrinth.Auth.v2.Login2FARequest,
	): Promise<Labrinth.Auth.v2.Login2FAResponse> {
		return this.client.request<Labrinth.Auth.v2.Login2FAResponse>(`/auth/login/2fa`, {
			api: 'labrinth',
			version: 2,
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Create a new account with a password
	 *
	 * @param data - Account creation data
	 * @returns Promise resolving to a session response
	 */
	public async createAccount(
		data: Labrinth.Auth.v2.CreateAccountRequest,
	): Promise<Labrinth.Auth.v2.CreateAccountResponse> {
		return this.client.request<Labrinth.Auth.v2.CreateAccountResponse>(`/auth/create`, {
			api: 'labrinth',
			version: 2,
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Begin a password reset flow by sending a recovery email
	 *
	 * @param data - The username/email and captcha challenge
	 */
	public async resetPasswordBegin(data: Labrinth.Auth.v2.ResetPasswordRequest): Promise<void> {
		return this.client.request(`/auth/password/reset`, {
			api: 'labrinth',
			version: 2,
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Change a user's password (via reset flow or with old password)
	 *
	 * @param data - The password change data
	 */
	public async changePassword(data: Labrinth.Auth.v2.ChangePasswordRequest): Promise<void> {
		return this.client.request(`/auth/password`, {
			api: 'labrinth',
			version: 2,
			method: 'PATCH',
			body: data,
		})
	}
}
