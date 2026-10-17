import type { AbstractModrinthClient } from '@modrinth/api-client'

import { classification, fallback } from './providers/classification'
import { content } from './providers/content'
import { discord } from './providers/discord'
import { forges } from './providers/forges'
import { github } from './providers/github'
import type { LinkInfo, LinkMatch, Request } from './types'

const providers = [github, forges, discord, content, classification]

export function identifyLink(value: string): LinkMatch {
	const url = new URL(value)
	for (const provider of providers) {
		const match = provider(url)
		if (match) return match
	}
	return fallback(url)
}

/** External enrichment requests run in the browser and never receive Modrinth credentials. */
export function linkInfoQueryOptions(
	url: string,
	match: LinkMatch,
	client: AbstractModrinthClient,
) {
	return {
		queryKey: ['project-review', 'link-info', url] as const,
		queryFn: async ({ signal }: { signal: AbortSignal }): Promise<LinkInfo> => {
			const request: Request = <T>(value: string) => {
				const endpoint = new URL(value)
				return client.request<T>(`${endpoint.pathname}${endpoint.search}`, {
					api: endpoint.origin,
					// A dot segment addresses unversioned APIs through the client's versioned URL builder.
					version: '.',
					skipAuth: true,
					headers: { 'Content-Type': '' },
					retry: false,
					timeout: 8000,
					signal,
				})
			}
			return match.load ? match.load(request, client) : { profiles: [], warnings: [] }
		},
		staleTime: 10 * 60 * 1000,
		gcTime: 30 * 60 * 1000,
		retry: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
	}
}
