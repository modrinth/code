import { FetchRequestHandler } from '../handlers/fetch-handler'
import { BaseRequestOptions, RequestHandler } from '../types/request'

export namespace Modrinth {
	/**
	 * Base API client providing a platform-agnostic `request` method.
	 *
	 * Consumers should extend this class to implement domain-specific modules
	 * (e.g., Projects, Versions). The underlying transport is delegated to a
	 * `RequestHandler`, which can be Nuxt, Fetch, or Tauri based, while the
	 * consumer-facing response shape remains `SimpleAsyncData`.
	 */
	export abstract class AbstractApiClient {
		protected readonly baseUrl: string
		protected readonly userAgent: string | undefined
		protected readonly requestHandler: RequestHandler
		/**
		 * @param requestHandler Transport implementation. Defaults to `FetchRequestHandler` if omitted.
		 * @param baseUrl Base URL for API requests. Defaults to `https://api.modrinth.com`.
		 * @param userAgent Optional user-agent header value forwarded by compatible handlers.
		 */
		constructor(requestHandler?: RequestHandler, baseUrl?: string, userAgent?: string | undefined) {
			this.baseUrl = baseUrl ?? 'https://api.modrinth.com'
			this.userAgent = userAgent
			this.requestHandler = requestHandler ?? new FetchRequestHandler(this.baseUrl, this.userAgent)
		}

		/**
		 * Perform a request via the configured `RequestHandler`.
		 *
		 * @typeParam DataT Data payload type
		 * @typeParam ErrorT Error payload type
		 * @param path Relative request path (e.g., `/v2/project/{id}`)
		 * @param options Cross-platform request options; Nuxt-only options will be respected by the Nuxt handler
		 * @returns A promise that resolves to `SimpleAsyncData<DataT, ErrorT>`
		 */
		protected request<DataT = any, ErrorT = unknown>(
			path: string,
			options?: BaseRequestOptions<DataT>,
		) {
			return this.requestHandler.request<DataT, ErrorT>(path, options)
		}
	}
}
