import { AbstractFeature, type FeatureConfig } from '../core/abstract-feature'
import type { RequestContext } from '../types/request'

export type VerboseLoggingConfig = FeatureConfig

export class VerboseLoggingFeature extends AbstractFeature {
	async execute<T>(next: () => Promise<T>, context: RequestContext): Promise<T> {
		const method = context.options.method ?? 'GET'
		const api = context.options.api
		const version = context.options.version
		const prefix = `[${method}] [${api}_v${version}]`

		console.log(`${prefix} ${context.url} SENT`)

		try {
			const result = await next()
			const size = JSON.stringify(result).length
			console.log(`${prefix} ${context.url} RECEIVED ${size} bytes`)
			return result
		} catch (error) {
			console.log(`${prefix} ${context.url} FAILED`)
			throw error
		}
	}
}
