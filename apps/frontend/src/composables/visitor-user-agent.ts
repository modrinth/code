import { tryUseNuxtApp } from '#imports'

// Nuxt's `useRequestHeaders` throws outside of a setup context, and `useBaseFetch` is
// called from plenty of places that no longer have one.
export function useVisitorUserAgent(): string | undefined {
	if (!import.meta.server) return undefined

	return tryUseNuxtApp()?.ssrContext?.event?.node?.req?.headers['user-agent']
}
