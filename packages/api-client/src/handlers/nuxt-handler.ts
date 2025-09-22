import { refGetter } from '../types/refs'
import { BaseRequestOptions, RequestHandler, SimpleAsyncData } from '../types/request'

/**
 * Nuxt request handler that wraps `useAsyncData` and `$fetch` at runtime.
 *
 * - Requires Nuxt 3 runtime globals: `useAsyncData` and `$fetch`.
 * - Applies `options.transform` within the handler for cross-env parity.
 * - Honors Nuxt options: `server`, `lazy`, `immediate`, `deep`, `dedupe`, `default`, `pick`, `watch`, `getCachedData`.
 * - Uses `options.key` or a derived key for de-duplication/caching.
 *
 * This class avoids compile-time coupling to Nuxt by resolving globals via `globalThis`.
 * If the globals are missing, it throws a clear runtime error.
 *  @remarks Requires running inside a Nuxt context.
 */
export class NuxtRequestHandler implements RequestHandler {
	constructor(
		private baseUrl: string,
		private userAgent?: string,
	) {}

	/**
	 * Perform a Nuxt `$fetch` via `useAsyncData` and expose a SimpleAsyncData wrapper.
	 *
	 * @typeParam DataT Data payload type
	 * @typeParam ErrorT Error payload type
	 * @param path Relative path to append to `baseUrl`
	 * @param options Nuxt-aware request options. Defaults: `server=true`, `lazy=false`, `immediate=true`, `deep=false`.
	 * @throws If Nuxt globals are not available at runtime
	 */
	async request<DataT = any, ErrorT = unknown>(
		path: string,
		options: BaseRequestOptions<DataT> = {},
	): Promise<SimpleAsyncData<DataT, ErrorT>> {
		// Resolve globals provided by Nuxt ($fetch, useAsyncData)
		const g: any = globalThis as any
		const useAsyncData = g.useAsyncData as
			| (<T>(key: string, handler: () => Promise<T>, opts?: any) => any)
			| undefined
		const $fetch = g.$fetch as (<T>(input: any, init?: any) => Promise<T>) | undefined

		if (!useAsyncData || !$fetch) {
			throw new Error(
				'NuxtRequestHandler requires Nuxt globals (useAsyncData, $fetch) to be available',
			)
		}

		const key = options.key ?? `${path}:${JSON.stringify(options.query ?? {})}`

		const nuxtAsync = await useAsyncData<DataT>(
			key,
			async () => {
				const result = await $fetch<DataT>(`${this.baseUrl}${path}`, {
					method: options.method ?? 'GET',
					headers: {
						...(this.userAgent ? { 'User-Agent': this.userAgent } : {}),
						...(options.headers ?? {}),
					},
					body: options.body,
					query: options.query,
				})
				if (options.transform) return await options.transform(result)
				return result
			},
			{
				server: options.server ?? true,
				lazy: options.lazy ?? false,
				immediate: options.immediate ?? true,
				deep: options.deep ?? false,
				dedupe: options.dedupe,
				default: options.default,
				pick: options.pick,
				watch: options.watch,
				getCachedData: options.getCachedData,
				transform: undefined, // we already apply transform inside handler for cross-env parity
			},
		)

		// Wrap Nuxt refs into SimpleAsyncData-compatible interface
		const simple: SimpleAsyncData<DataT, ErrorT> = {
			data: refGetter(() => nuxtAsync.data.value),
			error: refGetter(() => nuxtAsync.error.value),
			status: refGetter(() => nuxtAsync.status.value),
			refresh: nuxtAsync.refresh,
			execute: nuxtAsync.execute,
			clear: nuxtAsync.clear,
		}

		return simple
	}
}
