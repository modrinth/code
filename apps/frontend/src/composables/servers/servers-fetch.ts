import type { V1ErrorInfo } from '@modrinth/utils'
import { ModrinthServerError, ModrinthServersFetchError } from '@modrinth/utils'
import { $fetch, FetchError } from 'ofetch'

export interface ServersFetchOptions {
	method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'
	contentType?: string
	body?: Record<string, any>
	version?: number | 'internal'
	override?: {
		url?: string
		token?: string
	}
	retry?: number | boolean
	bypassAuth?: boolean
}

export async function useServersFetch<T>(
	path: string,
	options: ServersFetchOptions = {},
	module?: string,
	errorContext?: string,
): Promise<T> {
	const config = useRuntimeConfig()
	const auth = await useAuth()
	const authToken = auth.value?.token

	if (!authToken && !options.bypassAuth) {
		const error = new ModrinthServersFetchError(
			'[Modrinth Servers] Cannot fetch without auth',
			10000,
		)
		throw new ModrinthServerError('Missing auth token', 401, error, module)
	}

	const {
		method = 'GET',
		contentType = 'application/json',
		body,
		version = 0,
		override,
		retry = method === 'GET' ? 3 : 0,
	} = options

	const circuitBreakerKey = `${module || 'default'}_${path}`
	const failureCount = useState<number>(`fetch_failures_${circuitBreakerKey}`, () => 0)
	const lastFailureTime = useState<number>(`last_failure_${circuitBreakerKey}`, () => 0)

	const now = Date.now()
	if (failureCount.value >= 3 && now - lastFailureTime.value < 30000) {
		const error = new ModrinthServersFetchError(
			'[Modrinth Servers] Circuit breaker open - too many recent failures',
			503,
		)
		throw new ModrinthServerError('Service temporarily unavailable', 503, error, module)
	}

	if (now - lastFailureTime.value > 30000) {
		failureCount.value = 0
	}

	const base = (import.meta.server ? config.pyroBaseUrl : config.public.pyroBaseUrl)?.replace(
		/\/$/,
		'',
	)

	if (!base) {
		const error = new ModrinthServersFetchError(
			'[Modrinth Servers] Cannot fetch without base url. Make sure to set a PYRO_BASE_URL in environment variables',
			10001,
		)
		throw new ModrinthServerError('Configuration error: Missing PYRO_BASE_URL', 500, error, module)
	}

	const versionString = `v${version}`
	let newOverrideUrl = override?.url
	if (newOverrideUrl && newOverrideUrl.includes('v0') && version !== 0) {
		newOverrideUrl = newOverrideUrl.replace('v0', versionString)
	}

	const fullUrl = newOverrideUrl
		? `https://${newOverrideUrl}/${path.replace(/^\//, '')}`
		: version === 0
			? `${base}/modrinth/v${version}/${path.replace(/^\//, '')}`
			: version === 'internal'
				? `${base}/_internal/${path.replace(/^\//, '')}`
				: `${base}/v${version}/${path.replace(/^\//, '')}`

	const headers: Record<string, string> = {
		'User-Agent': 'Modrinth/1.0 (https://modrinth.com)',
		'X-Archon-Request': 'true',
		Vary: 'Accept, Origin',
	}

	if (!options.bypassAuth) {
		headers.Authorization = `Bearer ${override?.token ?? authToken}`
		headers['Access-Control-Allow-Headers'] = 'Authorization'
	}

	if (contentType !== 'none') {
		headers['Content-Type'] = contentType
	}

	if (import.meta.client && typeof window !== 'undefined') {
		headers.Origin = window.location.origin
	}

	let attempts = 0
	const maxAttempts = (typeof retry === 'boolean' ? (retry ? 3 : 1) : retry) + 1
	let lastError: Error | null = null

	while (attempts < maxAttempts) {
		try {
			const response = await $fetch<T>(fullUrl, {
				method,
				headers,
				body:
					body && contentType === 'application/json' ? JSON.stringify(body) : (body ?? undefined),
				timeout: 10000,
			})

			failureCount.value = 0
			return response
		} catch (error) {
			lastError = error as Error
			attempts++

			if (error instanceof FetchError) {
				const statusCode = error.response?.status
				const statusText = error.response?.statusText || 'Unknown error'

				if (statusCode && statusCode >= 500) {
					failureCount.value++
					lastFailureTime.value = now
				}

				let v1Error: V1ErrorInfo | undefined
				if (error.data?.error && error.data?.description) {
					v1Error = {
						context: errorContext,
						...error.data,
					}
				}

				const errorMessages: { [key: number]: string } = {
					400: 'Bad Request',
					401: 'Unauthorized',
					403: 'Forbidden',
					404: 'Not Found',
					405: 'Method Not Allowed',
					408: 'Request Timeout',
					429: "You're making requests too quickly. Please wait a moment and try again.",
					500: 'Internal Server Error',
					502: 'Bad Gateway',
					503: 'Service Unavailable',
					504: 'Gateway Timeout',
				}

				const message =
					statusCode && statusCode in errorMessages
						? errorMessages[statusCode]
						: `HTTP Error: ${statusCode || 'unknown'} ${statusText}`

				const isRetryable = statusCode ? [408, 429].includes(statusCode) : false
				const is5xxRetryable =
					statusCode && statusCode >= 500 && statusCode < 600 && method === 'GET' && attempts === 1

				if (!(isRetryable || is5xxRetryable) || attempts >= maxAttempts) {
					console.error('Fetch error:', error)

					const fetchError = new ModrinthServersFetchError(
						`[Modrinth Servers] ${error.message}`,
						statusCode,
						error,
					)
					throw new ModrinthServerError(
						`[Modrinth Servers] ${message}`,
						statusCode,
						fetchError,
						module,
						v1Error,
					)
				}

				const baseDelay = statusCode && statusCode >= 500 ? 5000 : 1000
				const delay = Math.min(baseDelay * Math.pow(2, attempts - 1) + Math.random() * 1000, 15000)
				console.warn(`Retrying request in ${delay}ms (attempt ${attempts}/${maxAttempts - 1})`)
				await new Promise((resolve) => setTimeout(resolve, delay))
				continue
			}

			console.error('Unexpected fetch error:', error)
			const fetchError = new ModrinthServersFetchError(
				'[Modrinth Servers] An unexpected error occurred during the fetch operation.',
				undefined,
				error as Error,
			)
			throw new ModrinthServerError(
				'Unexpected error during fetch operation',
				undefined,
				fetchError,
				module,
			)
		}
	}

	console.error('All retry attempts failed:', lastError)
	if (lastError instanceof FetchError) {
		const statusCode = lastError.response?.status
		const pyroError = new ModrinthServersFetchError(
			'Maximum retry attempts reached',
			statusCode,
			lastError,
		)
		throw new ModrinthServerError('Maximum retry attempts reached', statusCode, pyroError, module)
	}

	const fetchError = new ModrinthServersFetchError(
		'Maximum retry attempts reached',
		undefined,
		lastError || undefined,
	)
	throw new ModrinthServerError('Maximum retry attempts reached', undefined, fetchError, module)
}
