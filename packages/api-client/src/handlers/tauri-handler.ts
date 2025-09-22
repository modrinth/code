import { ref } from '../types/refs'
import { BaseRequestOptions, RequestHandler, SimpleAsyncData } from '../types/request'

/**
 * Tauri request handler using the Tauri global `window.__TAURI__.invoke`.
 *
 * - Avoids compile-time dependency on `@tauri-apps/api` by reading the global.
 * - Applies `options.transform` after a successful invocation.
 * - Honors `options.immediate` (default `true`).
 * - Accepts Nuxt options but ignores them.
 *
 * @experimental API surface may change as we iterate on Tauri integration.
 * @remarks Requires running inside a Tauri WebView context.
 */
export class TauriRequestHandler implements RequestHandler {
	/**
	 * @param command The backend command name to invoke via Tauri
	 * @default 'api_request'
	 */
	constructor(private command: string = 'api_request') {}

	/**
	 * Perform a Tauri `invoke` and expose results via SimpleAsyncData.
	 *
	 * @typeParam DataT Data payload type
	 * @typeParam ErrorT Error payload type
	 * @param path Logical path forwarded to the backend (your command decides how to use it)
	 * @param options Request options (method, headers, body, query, transform, immediate)
	 * @throws If Tauri runtime (`window.__TAURI__.invoke`) is not available
	 */
	async request<DataT = any, ErrorT = unknown>(
		path: string,
		options: BaseRequestOptions<DataT> = {},
	): Promise<SimpleAsyncData<DataT, ErrorT>> {
		const data = ref<DataT | undefined>(undefined)
		const error = ref<ErrorT | undefined>(undefined)
		const status = ref<'idle' | 'pending' | 'success' | 'error'>('idle')

		// Resolve invoke from Tauri global to avoid compile-time deps
		const tauriGlobal = (globalThis as any).__TAURI__
		const invoke: (<T>(cmd: string, args?: any) => Promise<T>) | undefined = tauriGlobal?.invoke
		if (!invoke) {
			throw new Error('TauriRequestHandler requires Tauri runtime (window.__TAURI__.invoke)')
		}

		const doInvoke = async () => {
			status.value = 'pending'
			error.value = undefined
			try {
				const result = await invoke!<DataT>(this.command, {
					path,
					method: options.method ?? 'GET',
					headers: options.headers,
					body: options.body,
					query: options.query,
				})
				const transformed = options.transform ? await options.transform(result) : result
				data.value = transformed
				status.value = 'success'
			} catch (e: any) {
				error.value = e as ErrorT
				status.value = 'error'
			}
		}

		if (options.immediate !== false) {
			await doInvoke()
		}

		return {
			data,
			error,
			status,
			refresh: async () => {
				await doInvoke()
			},
			execute: async () => {
				await doInvoke()
			},
			clear: () => {
				data.value = options.default ? (options.default() as any) : undefined
				error.value = undefined
				status.value = 'idle'
			},
		}
	}
}
