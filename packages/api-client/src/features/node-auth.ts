import { AbstractFeature, type FeatureConfig } from '../core/abstract-feature'
import { ModrinthApiError } from '../core/errors'
import type { RequestContext } from '../types/request'

/**
 * Node authentication credentials
 */
export interface NodeAuth {
	/** Node instance URL (e.g., "node-xyz.modrinth.com/modrinth/v0/fs") */
	url: string
	/** JWT token */
	token: string
}

export interface NodeAuthConfig extends FeatureConfig {
	/**
	 * Get current node auth. Returns null if not authenticated.
	 */
	getAuth: () => NodeAuth | null

	/**
	 * Refresh the node authentication token.
	 */
	refreshAuth: () => Promise<void>
}

/**
 * Handles authentication for Kyros node fs requests:
 * - Automatically injects Authorization header
 * - Builds the correct URL from node instance
 * - Handles 401 errors by refreshing and retrying (max 3 times)
 *
 * Only applies to requests with `useNodeAuth: true` in options.
 *
 * @example
 * ```typescript
 * const nodeAuth = new NodeAuthFeature({
 *   getAuth: () => nodeAuthState.getAuth?.() ?? null,
 *   refreshAuth: async () => {
 *     if (nodeAuthState.refreshAuth) {
 *       await nodeAuthState.refreshAuth()
 *     }
 *   },
 * })
 * client.addFeature(nodeAuth)
 * ```
 */
export class NodeAuthFeature extends AbstractFeature {
	declare protected config: NodeAuthConfig
	private refreshPromise: Promise<void> | null = null

	shouldApply(context: RequestContext): boolean {
		return context.options.useNodeAuth === true && this.config.enabled !== false
	}

	private async refreshAuthWithLock(): Promise<void> {
		if (this.refreshPromise) {
			return this.refreshPromise
		}
		this.refreshPromise = this.config.refreshAuth().finally(() => {
			this.refreshPromise = null
		})
		return this.refreshPromise
	}

	async execute<T>(next: () => Promise<T>, context: RequestContext): Promise<T> {
		const maxRetries = 3
		let retryCount = 0

		let auth = this.config.getAuth()
		if (!auth || this.isTokenExpired(auth.token)) {
			await this.refreshAuthWithLock()
			auth = this.config.getAuth()
		}
		if (!auth) {
			throw new Error('Failed to obtain node authentication')
		}

		this.applyAuth(context, auth)

		while (true) {
			try {
				return await next()
			} catch (error) {
				if (error instanceof ModrinthApiError && error.statusCode === 401) {
					retryCount++
					if (retryCount >= maxRetries) {
						throw new Error(
							`Node authentication failed after ${maxRetries} retries. Please re-authenticate.`,
						)
					}

					await this.refreshAuthWithLock()
					auth = this.config.getAuth()
					if (!auth) {
						throw new Error('Failed to refresh node authentication')
					}

					this.applyAuth(context, auth)
					continue
				}
				throw error
			}
		}
	}

	private applyAuth(context: RequestContext, auth: NodeAuth): void {
		const baseUrl = `https://${auth.url.replace('v0/fs', '')}`
		context.url = this.buildUrl(context.path, baseUrl, context.options.version)

		context.options.headers = {
			...context.options.headers,
			Authorization: `Bearer ${auth.token}`,
		}

		context.options.skipAuth = true
	}

	private buildUrl(path: string, baseUrl: string, version: number | 'internal' | string): string {
		const base = baseUrl.replace(/\/$/, '')
		let versionPath = ''
		if (version === 'internal') {
			versionPath = '/_internal'
		} else if (typeof version === 'number') {
			versionPath = `/v${version}`
		} else if (typeof version === 'string') {
			versionPath = `/${version}`
		}
		const cleanPath = path.startsWith('/') ? path : `/${path}`
		return `${base}${versionPath}${cleanPath}`
	}

	/**
	 * Check if a JWT token is expired or about to expire
	 * Refreshes proactively if expiring within next 10 seconds
	 */
	private isTokenExpired(token: string): boolean {
		try {
			const payload = JSON.parse(atob(token.split('.')[1]))
			if (!payload.exp) return false
			// refresh if expiring within 10 seconds
			const expiresAt = payload.exp * 1000
			return Date.now() >= expiresAt - 10000
		} catch {
			// cant decode, assume valid and let server decide
			return false
		}
	}
}
