import { ref } from '../types/refs'
import { BaseRequestOptions, RequestHandler, SimpleAsyncData } from '../types/request'

/**
 * Fetch-based request handler for browser and Node environments.
 *
 * - Uses the global `fetch` API and returns a `SimpleAsyncData` wrapper with
 *   `data`, `error`, `status`, `refresh`, `execute`, and `clear`.
 * - Applies `options.transform` after a successful JSON parse.
 * - Honors `options.immediate` (default `true`) to auto-run the request.
 * - Nuxt-only options in `BaseRequestOptions` are accepted but ignored.
 *
 * @example
 * const handler = new FetchRequestHandler('https://api.example.com', 'my-app/1.0')
 * const res = await handler.request<User>('/v2/me')
 * console.log(res.status.value, res.data.value)
 */
export class FetchRequestHandler implements RequestHandler {
	/**
	 * @param baseUrl Base URL prefix applied to all paths
	 * @param userAgent Optional user-agent header to send
	 */
	constructor(
		private baseUrl: string,
		private userAgent?: string,
	) {}

	/**
	 * Perform a fetch request and expose an AsyncData-like wrapper.
	 *
	 * @typeParam DataT Data payload type
	 * @typeParam ErrorT Error payload type
	 * @param path Relative path to append to `baseUrl`
	 * @param options Request options; `method` defaults to `'GET'`, `immediate` defaults to `true`.
	 * @returns A `SimpleAsyncData` wrapper with helpers to refresh/execute/clear
	 */
	async request<DataT = any, ErrorT = unknown>(
		path: string,
		options: BaseRequestOptions<DataT> = {},
	): Promise<SimpleAsyncData<DataT, ErrorT>> {
		const data = ref<DataT | undefined>(undefined)
		const error = ref<ErrorT | undefined>(undefined)
		const status = ref<'idle' | 'pending' | 'success' | 'error'>('idle')

		const buildUrl = () => {
			const url = new URL(`${this.baseUrl}${path}`)
			if (options.query) {
				for (const [k, v] of Object.entries(options.query)) {
					if (v !== undefined && v !== null) url.searchParams.append(k, String(v))
				}
			}
			return url.toString()
		}

		const doFetch = async () => {
			status.value = 'pending'
			error.value = undefined
			try {
				const res = await fetch(buildUrl(), {
					method: options.method ?? 'GET',
					headers: {
						'Content-Type': 'application/json',
						...(this.userAgent ? { 'User-Agent': this.userAgent } : {}),
						...(options.headers ?? {}),
					},
					body: options.body ? JSON.stringify(options.body) : undefined,
				})

				if (!res.ok) {
					const err: any = new Error(`HTTP ${res.status}: ${res.statusText}`)
					;(err.status = res.status), (err.statusText = res.statusText)
					throw err
				}

				let json = (await res.json()) as DataT
				if (options.transform) {
					json = await options.transform(json)
				}
				data.value = json
				status.value = 'success'
			} catch (e: any) {
				error.value = e as ErrorT
				status.value = 'error'
			}
		}

		const clear = () => {
			data.value = options.default ? (options.default() as any) : undefined
			error.value = undefined
			status.value = 'idle'
		}

		// immediate by default
		if (options.immediate !== false) {
			await doFetch()
		}

		return {
			data,
			error,
			status,
			refresh: async () => {
				await doFetch()
			},
			execute: async () => {
				await doFetch()
			},
			clear,
		}
	}
}
