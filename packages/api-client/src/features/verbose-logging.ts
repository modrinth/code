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
			console.debug(`${prefix} ${context.url} FAILED`)
			throw error
		}
	}
}
