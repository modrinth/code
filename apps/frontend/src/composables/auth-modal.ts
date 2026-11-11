import type { ComputedRef, InjectionKey } from 'vue'
import type { LocationQuery, RouteLocationNormalizedLoaded, RouteLocationRaw } from 'vue-router'

export type AuthLocation = {
	path: string
	fullPath: string
	hash: string
	query: LocationQuery
	name: string | null
}

export const EMBEDDABLE_AUTH_PATHS = [
	'/auth/sign-in',
	'/auth/sign-up',
	'/auth/reauthenticate',
	'/auth/verify-email',
] as const

export type EmbeddableAuthPath = (typeof EMBEDDABLE_AUTH_PATHS)[number]

const PAGE_ONLY_QUERY_KEYS = ['code', 'state', 'flow', 'launcher'] as const

export const authModalLocation = shallowRef<AuthLocation | null>(null)
export const authModalReady = ref(false)

export const AUTH_ROUTE_KEY: InjectionKey<ComputedRef<AuthLocation | null>> = Symbol('auth-route')

function queryHasValue(query: LocationQuery, key: string) {
	const value = query[key]
	if (Array.isArray(value)) {
		return value.some((entry) => entry != null && entry !== '')
	}
	return value != null && value !== ''
}

export function isEmbeddableAuthLocation(to: { path: string; query: LocationQuery }) {
	return (
		(EMBEDDABLE_AUTH_PATHS as readonly string[]).includes(to.path) &&
		!PAGE_ONLY_QUERY_KEYS.some((key) => queryHasValue(to.query, key))
	)
}

export function isAuthEmbedded() {
	return inject(AUTH_ROUTE_KEY, null) != null
}

export function openAuthModal(to: {
	path: string
	fullPath: string
	hash?: string
	query: LocationQuery
	name?: unknown
}) {
	authModalLocation.value = {
		path: to.path,
		fullPath: to.fullPath,
		hash: to.hash ?? '',
		query: { ...to.query },
		name: typeof to.name === 'string' ? to.name : null,
	}
}

export function closeAuthModal() {
	authModalLocation.value = null
}

export function useAuthRoute(): RouteLocationNormalizedLoaded {
	const override = inject(AUTH_ROUTE_KEY, null)
	const route = useRoute()
	if (!override) {
		return route
	}

	return new Proxy(route, {
		get(target, prop, receiver) {
			const source = override.value
			if (source && Object.prototype.hasOwnProperty.call(source, prop)) {
				return source[prop as keyof AuthLocation]
			}
			const value = Reflect.get(target, prop, receiver)
			return typeof value === 'function' ? value.bind(target) : value
		},
	})
}

export function openEmbeddableAuth(to: RouteLocationRaw) {
	const resolved = useRouter().resolve(to)
	if (!authModalReady.value || !isEmbeddableAuthLocation(resolved)) {
		return false
	}
	openAuthModal(resolved)
	return true
}

export function promptAuth(to: RouteLocationRaw) {
	const path = import.meta.client ? window.location.pathname : useRouter().resolve(to).path
	if (!path.startsWith('/auth') && openEmbeddableAuth(to)) {
		return Promise.resolve()
	}
	return navigateTo(to)
}

export function useAuthNavigation() {
	const embedded = inject(AUTH_ROUTE_KEY, null)

	return {
		async go(to: RouteLocationRaw) {
			if (embedded?.value && openEmbeddableAuth(to)) {
				return
			}
			await navigateTo(to)
		},
	}
}

export function useAuthPageTitle(title: () => string) {
	if (isAuthEmbedded()) {
		return
	}
	useHead({
		title: () => `${title()} - Modrinth`,
	})
}

export async function finishAuth(destination?: string | null, replace = false) {
	const current = import.meta.client
		? `${window.location.pathname}${window.location.search}${window.location.hash}`
		: ''
	closeAuthModal()
	if (!destination || destination === current) {
		return
	}
	await navigateTo(destination, replace ? { replace: true } : undefined)
}
