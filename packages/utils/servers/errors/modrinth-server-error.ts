import { FetchError } from 'ofetch'

import type { V1ErrorInfo } from '../types'

export class ModrinthServerError extends Error {
	constructor(
		message: string,
		public readonly statusCode?: number,
		public readonly originalError?: Error,
		public readonly module?: string,
		public readonly v1Error?: V1ErrorInfo,
		public readonly responseData?: unknown,
	) {
		let errorMessage = message
		let method = 'GET'
		let path = ''

		if (originalError instanceof FetchError) {
			const matches = message.match(/\[([A-Z]+)\]\s+"([^"]+)":/)
			if (matches) {
				method = matches[1]
				path = matches[2].replace(/https?:\/\/[^/]+\/[^/]+\/v\d+\//, '')
			}

			const statusMessage = (() => {
				if (!statusCode) return 'Unknown Error'
				switch (statusCode) {
					case 400:
						return 'Bad Request'
					case 401:
						return 'Unauthorized'
					case 403:
						return 'Forbidden'
					case 404:
						return 'Not Found'
					case 408:
						return 'Request Timeout'
					case 429:
						return 'Too Many Requests'
					case 500:
						return 'Internal Server Error'
					case 502:
						return 'Bad Gateway'
					case 503:
						return 'Service Unavailable'
					case 504:
						return 'Gateway Timeout'
					default:
						return `HTTP ${statusCode}`
				}
			})()

			errorMessage = `[${method}] ${statusMessage} (${statusCode}) while fetching ${path}${module ? ` in ${module}` : ''}`
		} else {
			errorMessage = `${message}${statusCode ? ` (${statusCode})` : ''}${module ? ` in ${module}` : ''}`
		}

		super(errorMessage)
		this.name = 'ModrinthServersFetchError'
	}
}
