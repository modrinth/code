import { AbstractFeature, type FeatureConfig } from '../core/abstract-feature'
import type { RequestContext } from '../types/request'

export type VerboseLoggingConfig = FeatureConfig

export class VerboseLoggingFeature extends AbstractFeature {
	async execute<T>(next: () => Promise<T>, context: RequestContext): Promise<T> {
		const method = context.options.method ?? 'GET'
		const api = context.options.api
		const version = context.options.version
		const prefix = `[${method}] [${api}_v${version}]`

		console.debug(`${prefix} ${context.url} SENT`)

		try {
			const result = await next()
			try {
				const size = result ? JSON.stringify(result).length : 0
				console.debug(`${prefix} ${context.url} RECEIVED ${size} bytes`)
			} catch {
				// ignore size calc fail
				console.debug(`${prefix} ${context.url} RECEIVED`)
			}
			return result
		} catch (error) {
			const details = formatErrorDetails(error)
			console.debug(`${prefix} ${context.url} FAILED${details ? ` — ${details}` : ''}`)
			throw error
		}
	}
}

function formatErrorDetails(error: unknown): string {
	if (!error || typeof error !== 'object') {
		return typeof error === 'string' ? error : ''
	}

	const err = error as {
		status?: number
		statusCode?: number
		statusText?: string
		message?: string
		data?: unknown
		responseData?: unknown
		originalError?: unknown
		response?: { status?: number; statusText?: string; _data?: unknown }
	}

	const status = err.status ?? err.statusCode ?? err.response?.status
	const statusText = err.statusText ?? err.response?.statusText
	const data = err.responseData ?? err.data ?? err.response?._data

	const parts: string[] = []
	if (status !== undefined) {
		parts.push(statusText ? `${status} ${statusText}` : String(status))
	}
	if (data !== undefined) {
		parts.push(`body: ${safeStringify(data)}`)
	} else if (err.message) {
		parts.push(err.message)
	}
	return parts.join(' ')
}

function safeStringify(value: unknown): string {
	if (typeof value === 'string') return value
	try {
		return JSON.stringify(value)
	} catch {
		return String(value)
	}
}
