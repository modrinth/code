import { SimpleRef } from './refs'

/**
 * Minimal request lifecycle status compatible with Nuxt's AsyncData statuses.
 * - `idle`: request has not started yet
 * - `pending`: request is in-flight
 * - `success`: request finished successfully
 * - `error`: request failed
 */
export type AsyncDataRequestStatus = 'idle' | 'pending' | 'success' | 'error'

export type HttpMethod = 'GET' | 'OPTIONS' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'

/**
 * Platform-agnostic AsyncData-like shape.
 *
 * Handlers should implement this to provide a unified interface regardless of
 * the underlying platform:
 * - Nuxt: wraps `useAsyncData` refs and methods
 * - Browser/Node: wraps fetch results in in-memory refs and functions
 * - Tauri: wraps `invoke` results in in-memory refs and functions
 *
 * Methods `refresh` and `execute` are equivalent here and should re-run the
 * last request with the same inputs.
 *
 * @template DataT Data payload type
 * @template ErrorT Error payload type
 */
export interface SimpleAsyncData<DataT, ErrorT = unknown> {
	data: SimpleRef<DataT | undefined>
	error: SimpleRef<ErrorT | undefined>
	status: SimpleRef<AsyncDataRequestStatus>
	refresh: (opts?: { dedupe?: 'cancel' | 'defer' }) => Promise<void>
	execute: (opts?: { dedupe?: 'cancel' | 'defer' }) => Promise<void>
	clear: () => void
}

/**
 * Transform function used to post-process successful responses.
 *
 * Handlers should apply `transform` after receiving the response data and
 * before updating `data.value`.
 *
 * @template DataT Data payload type
 */
export type TransformFn<DataT> = (input: DataT) => DataT | Promise<DataT>

/**
 * Options common across environments and a subset that is Nuxt-specific.
 *
 * Non-Nuxt handlers (e.g. fetch/tauri) may ignore Nuxt-only options such as
 * `server`, `lazy`, or `watch`, but they are accepted for a uniform API.
 *
 * @template DataT Data payload type for defaults/transform/pick
 */
export interface BaseRequestOptions<DataT = any> {
	/** HTTP method to use
	 * @default 'GET'
	 */
	method?: HttpMethod
	/** Additional headers to send with the request */
	headers?: Record<string, string>
	/** Request body (will usually be JSON-serialized by handlers) */
	body?: any
	/** Query parameters appended to the URL */
	query?: Record<string, any>
	/** Unique key primarily used by Nuxt handlers for caching/de-duplication */
	key?: string

	/** Whether to fetch on server in Nuxt SSR mode
	 * @only Nuxt
	 * @default true
	 */
	server?: boolean

	/** Fetch lazily after route navigation instead of blocking
	 * @only Nuxt
	 * @default false
	 */
	lazy?: boolean

	/** Start the request immediately
	 * @only Nuxt
	 * @default true
	 */
	immediate?: boolean
	/** Return data in a deep ref
	 * @only Nuxt
	 * @default false
	 */
	deep?: boolean
	/** Avoid fetching same key more than once
	 * @only Nuxt
	 * @default 'cancel'
	 */
	dedupe?: 'cancel' | 'defer'
	/** Default value factory for data before resolution
	 * @only Nuxt
	 */
	default?: () => DataT | SimpleRef<DataT> | null
	/** Post-process the resolved data */
	transform?: TransformFn<DataT>
	/** Only pick these keys from the result
	 * @only Nuxt
	 */
	pick?: string[]
	/** Reactive sources to auto-refresh on change
	 * @only Nuxt
	 * @default false
	 */
	watch?: any[] | false
	/** Custom cache read for Nuxt async data
	 * @only Nuxt
	 */
	getCachedData?: (key: string, ctx?: unknown) => DataT | undefined
}

/**
 * Unified platform-agnostic request interface.
 *
 * Implementations should perform the request at `path` relative to their own
 * base URL/runtime and return a `SimpleAsyncData` wrapper. The wrapper enables
 * consumers to use a consistent API for data, status, errors and re-execution.
 */
export interface RequestHandler {
	/**
	 * Perform a request and return an AsyncData-like wrapper.
	 *
	 * @template DataT Data payload type
	 * @template ErrorT Error payload type
	 * @param path Relative request path (e.g. `/v2/project/abc`)
	 * @param options Cross-platform request options and Nuxt-specific options
	 * @returns A promise that resolves to a SimpleAsyncData wrapper
	 */
	request<DataT = any, ErrorT = unknown>(
		path: string,
		options?: BaseRequestOptions<DataT>,
	): Promise<SimpleAsyncData<DataT, ErrorT>>
}
