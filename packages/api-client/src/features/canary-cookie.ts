import { AbstractFeature, type FeatureConfig } from '../core/abstract-feature'

export const LABRINTH_CANARY_COOKIE = 'labrinth-canary=always'

export interface CanaryCookieConfig extends FeatureConfig {
	getCookie?: () => string | undefined | Promise<string | undefined>
}

export class CanaryCookieFeature extends AbstractFeature {
	declare protected config: CanaryCookieConfig

	constructor(config?: CanaryCookieConfig) {
		super(config)
	}

	shouldApply(context: Parameters<AbstractFeature['shouldApply']>[0]): boolean {
		return super.shouldApply(context) && context.options.api === 'labrinth'
	}

	async execute<T>(next: () => Promise<T>, context: Parameters<AbstractFeature['execute']>[1]) {
		const cookie = this.config.getCookie ? await this.config.getCookie() : LABRINTH_CANARY_COOKIE
		if (!cookie) {
			return next()
		}

		const headers = { ...(context.options.headers ?? {}) }
		const existingCookie = headers.cookie ?? headers.Cookie

		if (!existingCookie?.split('; ').includes(cookie)) {
			headers.cookie = existingCookie ? `${existingCookie}; ${cookie}` : cookie
			delete headers.Cookie
			context.options.headers = headers
		}

		return next()
	}
}
