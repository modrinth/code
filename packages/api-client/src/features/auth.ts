import { AbstractFeature, type FeatureConfig } from '../core/abstract-feature'
import type { RequestContext } from '../types/request'

/**
 * Authentication feature configuration
 */
export interface AuthConfig extends FeatureConfig {
	/**
	 * Authentication token
	 * - string: static token
	 * - function: async function that returns token (useful for dynamic tokens)
	 */
	token: string | (() => Promise<string | undefined>)

	/**
	 * Token prefix (e.g., 'Bearer', 'Token')
	 * @default 'Bearer'
	 */
	tokenPrefix?: string

	/**
	 * Custom header name for the token
	 * @default 'Authorization'
	 */
	headerName?: string
}

/**
 * Authentication feature
 *
 * Automatically injects authentication tokens into request headers.
 * Supports both static tokens and dynamic token providers.
 *
 * @example
 * ```typescript
 * const auth = new AuthFeature({
 *   token: async () => process.env.MODRINTH_TOKEN
 * })
 * ```
 */
export class AuthFeature extends AbstractFeature {
	declare protected config: AuthConfig

	async execute<T>(next: () => Promise<T>, context: RequestContext): Promise<T> {
		const token = await this.getToken()

		if (token) {
			const headerName = this.config.headerName ?? 'Authorization'
			const tokenPrefix = this.config.tokenPrefix ?? 'Bearer'
			const headerValue = tokenPrefix ? `${tokenPrefix} ${token}` : token

			context.options.headers = {
				...context.options.headers,
				[headerName]: headerValue,
			}
		}

		return next()
	}

	shouldApply(context: RequestContext): boolean {
		if (context.options.skipAuth) {
			return false
		}

		// Skip if Authorization header is already explicitly set
		const headerName = this.config.headerName ?? 'Authorization'
		if (context.options.headers?.[headerName]) {
			return false
		}

		return super.shouldApply(context)
	}

	/**
	 * Get the authentication token
	 *
	 * Handles both static tokens and async token providers
	 */
	private async getToken(): Promise<string | undefined> {
		const { token } = this.config

		if (typeof token === 'function') {
			return await token()
		}

		return token
	}
}
