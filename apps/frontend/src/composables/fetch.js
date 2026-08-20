/**
 * @deprecated Use `@modrinth/api-client` via `injectModrinthClient()` instead.
 * This composable is kept for legacy code that hasn't been migrated yet.
 */

import { useVisitorUserAgent } from '~/composables/visitor-user-agent.ts'
import { withLabrinthCanaryHeader } from '~/helpers/canary.ts'
import { readEnv } from '~/helpers/env.ts'
import { getFrontendUserAgent, VISITOR_USER_AGENT_HEADER } from '~/helpers/user-agent.ts'

let cachedRateLimitKey = undefined
let rateLimitKeyPromise = undefined

async function getRateLimitKey(config) {
	if (config.rateLimitKey) return config.rateLimitKey
	if (cachedRateLimitKey !== undefined) return cachedRateLimitKey

	if (!rateLimitKeyPromise) {
		rateLimitKeyPromise = readEnv('RATE_LIMIT_IGNORE_KEY')
	}

	cachedRateLimitKey = await rateLimitKeyPromise
	return cachedRateLimitKey
}

export const useBaseFetch = async (url, options = {}, skipAuth = false) => {
	const config = useRuntimeConfig()
	let base = import.meta.server ? config.apiBaseUrl : config.public.apiBaseUrl

	if (!options.headers) {
		options.headers = {}
	}

	options.headers = withLabrinthCanaryHeader(options.headers)

	if (import.meta.server) {
		options.headers['x-ratelimit-key'] = await getRateLimitKey(config)
		options.headers['User-Agent'] = getFrontendUserAgent(config.public.hash)

		const visitorUserAgent = useVisitorUserAgent()
		if (visitorUserAgent) {
			options.headers[VISITOR_USER_AGENT_HEADER] = visitorUserAgent
		}
	}

	if (!skipAuth) {
		const auth = await useAuth()

		options.headers.Authorization = auth.value.token
	}

	if (options.apiVersion || options.internal) {
		// Base may end in /vD/ or /vD. We would need to replace the digit with the new version number
		// and keep the trailing slash if it exists
		const baseVersion = base.match(/\/v\d\//)

		const replaceStr = options.internal ? `/_internal/` : `/v${options.apiVersion}/`

		if (baseVersion) {
			base = base.replace(baseVersion[0], replaceStr)
		} else {
			base = base.replace(/\/v\d$/, replaceStr)
		}

		delete options.apiVersion
	}

	return await $fetch(`${base}${url}`, {
		timeout: import.meta.server ? 10000 : undefined,
		...options,
	})
}
