import { AbstractFeature } from '../core/abstract-feature'
import type { RequestContext } from '../types/request'

export const PANEL_VERSION = 1

export class PanelVersionFeature extends AbstractFeature {
	async execute<T>(next: () => Promise<T>, context: RequestContext): Promise<T> {
		context.options.headers = {
			...context.options.headers,
			'X-Panel-Version': String(PANEL_VERSION),
		}
		return next()
	}

	shouldApply(_: RequestContext): boolean {
		return true
	}
}
