import { AbstractFeature, type FeatureConfig } from '../core/abstract-feature'
import type { RequestContext } from '../types/request'

export interface SelfHostedConfig extends FeatureConfig {
	baseUrl: string
	/** Whether a URL is on a paired Core. Those requests carry the account token; Core checks it with the backend. */
	isCoreUrl?: (url: string) => boolean
	token: () => Promise<string | undefined>
}

/** Routes private accounts and sharing without sending their sessions to public content services. */
export class SelfHostedFeature extends AbstractFeature {
	declare protected config: SelfHostedConfig

	constructor(config: SelfHostedConfig) {
		super(config)
	}

	async execute<T>(next: () => Promise<T>, context: RequestContext): Promise<T> {
		await routeSelfHostedRequest(context, this.config)
		return next()
	}
}

/** Shared by the typed client and the website's legacy fetch adapter. */
export async function routeSelfHostedRequest(
	context: Pick<RequestContext, 'url' | 'path' | 'options'>,
	config: SelfHostedConfig,
): Promise<void> {
	if (
		context.options.api === 'labrinth' &&
		context.path.split('?')[0] === '/image' &&
		(context.options.params?.context ?? new URL(context.url).searchParams.get('context')) ===
			'report'
	) {
		throw new Error('Report attachments are not supported by this account service yet.')
	}
	const privateRequest =
		context.options.api === 'sharedinstances' ||
		(context.options.api === 'labrinth' &&
			context.options.accountSource !== 'modrinth' &&
			isAccountPath(context.path))
	const hostingRequest = config.isCoreUrl?.(context.url) ?? false
	const skipAuth = context.options.skipAuth
	context.options.skipAuth = true
	const headers = new Headers(context.options.headers)
	if (privateRequest || hostingRequest) {
		if (privateRequest) {
			const source = new URL(context.url)
			context.url = new URL(source.pathname + source.search, config.baseUrl).toString()
		}
		if (!headers.has('authorization') && !skipAuth) {
			const token = await config.token()
			if (token) headers.set('authorization', `Bearer ${token}`)
		}
	} else {
		headers.delete('authorization')
	}
	const normalizedHeaders: Record<string, string> = {}
	headers.forEach((value, key) => {
		normalizedHeaders[key] = value
	})
	context.options.headers = normalizedHeaders
}

function isAccountPath(path: string): boolean {
	const pathname = path.split('?')[0] ?? path
	if (/^\/(?:_internal\/)?admin\/user(?:\/|$)/.test(pathname)) return true
	if (['/user_email', '/user_discord'].includes(pathname)) return true
	return /^\/(auth|globals|session|sessions|friend|friends|block|blocks|notification|notifications|pat|user|users|cores)(\/|$)/.test(
		pathname,
	)
}
