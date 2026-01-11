import { type NuxtClientConfig, NuxtModrinthClient } from '@modrinth/api-client'
import type { H3Event } from 'h3'

async function getRateLimitKeyFromSecretsStore(): Promise<string | undefined> {
	try {
		const mod = 'cloudflare:workers'
		const { env } = await import(/* @vite-ignore */ mod)
		return await env.RATE_LIMIT_IGNORE_KEY?.get()
	} catch {
		return undefined
	}
}

export function useServerModrinthClient(event: H3Event): NuxtModrinthClient {
	const config = useRuntimeConfig(event)
	const apiBaseUrl = (config.apiBaseUrl || config.public.apiBaseUrl).replace('/v2/', '/')

	const clientConfig: NuxtClientConfig = {
		labrinthBaseUrl: apiBaseUrl,
		rateLimitKey: config.rateLimitKey || getRateLimitKeyFromSecretsStore,
		features: [],
	}

	return new NuxtModrinthClient(clientConfig)
}
