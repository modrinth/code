import { AbstractModule } from '../../../core/abstract-module.js'
import type { Labrinth } from '../types.js'

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
	 * Validate email/password inputs for account creation without creating an account.
	 *
	 * @param data - Prospective account credentials
	 */
	public async validateCreateAccount(
		data: Labrinth.Auth.v2.ValidateCreateAccountRequest,
	): Promise<void> {
		return this.client.request(`/auth/create/validate`, {
			api: 'labrinth',
			version: 2,
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Create a new account from an OAuth callback flow state
	 *
	 * @param data - OAuth account creation data
	 * @returns Promise resolving to a session response
	 */
	public async createOAuthAccount(
		data: Labrinth.Auth.v2.CreateOAuthAccountRequest,
	): Promise<Labrinth.Auth.v2.CreateOAuthAccountResponse> {
		return this.client.request<Labrinth.Auth.v2.CreateOAuthAccountResponse>(`/auth/create/oauth`, {
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

	/**
	 * List the current user's registered passkeys
	 *
	 * @returns A promise that resolves to a list of the user's registered passkeys
	 */
	public async listPasskeys(): Promise<Labrinth.Auth.v2.Passkey[]> {
		return this.client.request<Labrinth.Auth.v2.Passkey[]>(`/auth/passkey`, {
			api: 'labrinth',
			version: 2,
			method: 'GET',
		})
	}

	/**
	 * Begin registering a new passkey, returning the WebAuthn creation options and a flow
	 *
	 * @returns A promise that resolves to the WebAuthn creation options and flow
	 */
	public async registerPasskeyStart(): Promise<Labrinth.Auth.v2.PasskeyRegisterStartResponse> {
		return this.client.request<Labrinth.Auth.v2.PasskeyRegisterStartResponse>(
			`/auth/passkey/register/start`,
			{
				api: 'labrinth',
				version: 2,
				method: 'POST',
			},
		)
	}

	/**
	 * Complete passkey registration with the created credential
	 *
	 * @param data The credential data and flow to complete registration with
	 * @returns A promise that resolves to the newly registered passkey
	 */
	public async registerPasskeyFinish(
		data: Labrinth.Auth.v2.PasskeyRegisterFinishRequest,
	): Promise<Labrinth.Auth.v2.Passkey> {
		return this.client.request<Labrinth.Auth.v2.Passkey>(`/auth/passkey/register/finish`, {
			api: 'labrinth',
			version: 2,
			method: 'POST',
			body: data,
		})
	}

	/**
	 * Begin a passkey authentication flow, returning the WebAuthn request options and a flow
	 *
	 * @returns A promise that resolves to the WebAuthn request options and a flow
	 */
	public async authenticatePasskeyStart(): Promise<Labrinth.Auth.v2.PasskeyAuthenticateStartResponse> {
		return this.client.request<Labrinth.Auth.v2.PasskeyAuthenticateStartResponse>(
			`/auth/passkey/start`,
			{
				api: 'labrinth',
				version: 2,
				method: 'POST',
				skipAuth: true,
			},
		)
	}

	/**
	 * Complete a passkey authentication flow, returning the new session
	 *
	 * @param data The credential data and flow to complete authentication with
	 * @returns A promise that resolves to the new session
	 */
	public async authenticatePasskeyFinish(
		data: Labrinth.Auth.v2.PasskeyAuthenticateFinishRequest,
	): Promise<Labrinth.Sessions.v2.Session> {
		return this.client.request<Labrinth.Sessions.v2.Session>(`/auth/passkey/finish`, {
			api: 'labrinth',
			version: 2,
			method: 'POST',
			body: data,
			skipAuth: true,
		})
	}

	/**
	 * Rename a passkey
	 *
	 * @param id The ID of the passkey to rename
	 * @param data The new name for the passkey
	 */
	public async renamePasskey(
		id: string,
		data: Labrinth.Auth.v2.PasskeyRenameRequest,
	): Promise<void> {
		return this.client.request(`/auth/passkey/${id}`, {
			api: 'labrinth',
			version: 2,
			method: 'PATCH',
			body: data,
		})
	}

	/**
	 * Delete a passkey
	 *
	 * @param id The ID of the passkey to delete
	 */
	public async deletePasskey(id: string): Promise<void> {
		return this.client.request(`/auth/passkey/${id}`, {
			api: 'labrinth',
			version: 2,
			method: 'DELETE',
		})
	}
}
