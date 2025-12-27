import { ModrinthApiError } from '../core/errors'

/**
 * Wrap a function with JWT retry logic.
 * On 401, calls refreshToken() and retries once.
 */
export async function withJWTRetry<T>(
	fn: () => Promise<T>,
	refreshToken: () => Promise<void>,
): Promise<T> {
	try {
		return await fn()
	} catch (error) {
		if (error instanceof ModrinthApiError && error.statusCode === 401) {
			await refreshToken()
			return await fn()
		}
		throw error
	}
}
