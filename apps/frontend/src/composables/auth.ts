import type { Labrinth } from '@modrinth/api-client'
import { useStorage } from '@vueuse/core'
import type { LocationQueryValue, RouteLocationNormalizedLoaded } from 'vue-router'

import type { CookieOptions } from '#app'

type AuthState = {
	user: Labrinth.Users.v2.User | null
	token: string
}

type QueryValue = LocationQueryValue | LocationQueryValue[] | undefined
type FullPathRoute = Pick<RouteLocationNormalizedLoaded, 'fullPath'>
type LauncherRoute = Pick<RouteLocationNormalizedLoaded, 'query'>

export const LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY = 'auth-last-sign-in-oauth-provider'
export const PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY = 'auth-pending-sign-in-oauth-provider'

const AUTH_COOKIE_OPTIONS = {
	maxAge: 60 * 60 * 24 * 365 * 10,
	sameSite: 'lax',
	secure: true,
	httpOnly: false,
	path: '/',
} satisfies CookieOptions<string | null>

const getQueryString = (value: QueryValue) => {
	if (Array.isArray(value)) {
		return value[0] ?? null
	}
	return value ?? null
}

export const useAuth = async (oldToken: string | null | undefined = null) => {
	const auth = useState<AuthState>('auth', () => ({
		user: null,
		token: '',
	}))

	if (!auth.value.user || oldToken) {
		auth.value = await initAuth(oldToken)
	}

	return auth
}

export const initAuth = async (oldToken: string | null | undefined = null) => {
	const auth: AuthState = {
		user: null,
		token: '',
	}

	if (oldToken === 'none') {
		return auth
	}

	const route = useRoute()
	const authCookie = useCookie<string | null>('auth-token', AUTH_COOKIE_OPTIONS)
	const authCode = getQueryString(route.query.code)

	if (oldToken) {
		authCookie.value = oldToken
	}

	if (authCode) {
		authCookie.value = authCode
	}

	if (authCookie.value) {
		auth.token = authCookie.value

		if (!auth.token || !auth.token.startsWith('mra_')) {
			return auth
		}

		try {
			auth.user = (await useBaseFetch(
				'user',
				{
					headers: {
						Authorization: auth.token,
					},
				},
				true,
			)) as Labrinth.Users.v2.User
		} catch {
			/* empty */
		}
	}

	if (!auth.user && auth.token) {
		try {
			const session = (await useBaseFetch(
				'session/refresh',
				{
					method: 'POST',
					headers: {
						Authorization: auth.token,
					},
				},
				true,
			)) as { session: string }

			auth.token = session.session
			authCookie.value = auth.token

			auth.user = (await useBaseFetch(
				'user',
				{
					headers: {
						Authorization: auth.token,
					},
				},
				true,
			)) as Labrinth.Users.v2.User
		} catch {
			authCookie.value = null
		}
	}

	return auth
}

export const getSignInRedirectPath = (route: FullPathRoute) => {
	const fullPath = route.fullPath
	if (fullPath === '/auth' || fullPath.startsWith('/auth/')) {
		return '/dashboard'
	}
	return fullPath
}

export const getSignInRouteObj = (route: FullPathRoute, redirectOverride?: string | null) => ({
	path: '/auth/sign-in',
	query: {
		redirect: redirectOverride ?? getSignInRedirectPath(route),
	},
})

export const getAuthUrl = (provider: string, redirect = '/dashboard') => {
	const config = useRuntimeConfig()
	const route = useNativeRoute()
	const launcher = getQueryString(route.query.launcher)

	const fullURL = launcher
		? (() => {
				const callbackUrl = new URL('/auth/sign-in', config.public.siteUrl)
				callbackUrl.searchParams.set('launcher', launcher)

				const ipver = getQueryString(route.query.ipver)
				const port = getQueryString(route.query.port)

				if (ipver) {
					callbackUrl.searchParams.set('ipver', ipver)
				}

				if (port) {
					callbackUrl.searchParams.set('port', port)
				}

				return callbackUrl.toString()
			})()
		: `${config.public.siteUrl}/auth/sign-in?redirect=${encodeURIComponent(redirect)}`

	return `${config.public.apiBaseUrl}auth/init?provider=${provider}&url=${encodeURIComponent(fullURL)}`
}

export const promotePendingSignInOAuthProvider = () => {
	if (!import.meta.client) return
	const pending = useStorage<string | null>(
		PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY,
		null,
		undefined,
		{ initOnMounted: true },
	)
	if (!pending.value) return
	const last = useStorage<string | null>(LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY, null, undefined, {
		initOnMounted: true,
	})
	last.value = pending.value
	pending.value = null
}

export const removeAuthProvider = async (provider: string) => {
	startLoading()

	const auth = await useAuth()

	await useBaseFetch('auth/provider', {
		method: 'DELETE',
		body: {
			provider,
		},
	})

	await useAuth(auth.value.token)

	stopLoading()
}

export const getLauncherRedirectUrl = (route: LauncherRoute) => {
	const ipver = getQueryString(route.query.ipver)
	const port = Number(getQueryString(route.query.port))
	const usesLocalhostRedirectionScheme = ['4', '6'].includes(ipver ?? '') && port < 65536

	return usesLocalhostRedirectionScheme
		? `http://${ipver === '4' ? '127.0.0.1' : '[::1]'}:${port}`
		: 'https://launcher-files.modrinth.com'
}
