import type { Labrinth } from '@modrinth/api-client'
import { useStorage } from '@vueuse/core'
import type { RouteLocationNormalizedLoaded } from 'vue-router'

import { rememberStoredAccount } from '@/composables/accounts.ts'
import { useAuthCookie } from '@/composables/auth-cookie.ts'
import {
	getQueryString,
	isLauncherProtocolV2,
	LAUNCHER_APP_SESSION_QUERY_PARAM,
	LAUNCHER_AUTH_QUERY_PARAMS,
} from '@/composables/launcher-auth.ts'

type AuthState = {
	user: Labrinth.Users.v2.User | null
	token: string
}

type FullPathRoute = Pick<RouteLocationNormalizedLoaded, 'fullPath'>
type AuthInitRoute = Pick<RouteLocationNormalizedLoaded, 'fullPath' | 'path' | 'query'>

const normalizeAuthToken = (value: unknown) => {
	if (typeof value === 'string') {
		return value
	}
	return ''
}

const getErrorStatus = (error: unknown): number | undefined => {
	if (!error || typeof error !== 'object') {
		return undefined
	}

	const typedError = error as { statusCode?: unknown; status?: unknown }
	const status = typedError.statusCode ?? typedError.status

	return typeof status === 'number' ? status : undefined
}

// only when labrinth actually gives us an auth error
const isAuthFailure = (error: unknown): boolean => {
	const status = getErrorStatus(error)
	return status === 401 || status === 403
}

const clearAuthCookie = (auth: AuthState, authCookie: { value: string | null }) => {
	authCookie.value = null
	auth.token = ''
	auth.user = null
}

export const useAuthState = () =>
	useState<AuthState>('auth', () => ({
		user: null,
		token: '',
	}))

export const useAuth = async (
	oldToken: string | null | undefined = null,
	route?: AuthInitRoute,
) => {
	const auth = useAuthState()

	if (!auth.value.user || oldToken) {
		auth.value = await initAuth(oldToken, route)
	}

	return auth
}

export const initAuth = async (
	oldToken: string | null | undefined = null,
	route?: AuthInitRoute,
) => {
	const auth: AuthState = {
		user: null,
		token: '',
	}

	if (oldToken === 'none') {
		return auth
	}

	const resolvedRoute = route ?? useRoute()
	const authCookie = useAuthCookie()

	if (oldToken) {
		const normalized = normalizeAuthToken(oldToken)
		if (normalized) {
			authCookie.value = normalized
		}
	}

	const oauthCode = normalizeAuthToken(resolvedRoute.query.code)
	const keepsExistingLauncherSession = isLauncherProtocolV2(resolvedRoute)
	if (
		oauthCode &&
		!keepsExistingLauncherSession &&
		!resolvedRoute.fullPath.includes('new_account=true')
	) {
		authCookie.value = oauthCode
	}

	if (
		resolvedRoute.fullPath.includes('new_account=true') &&
		resolvedRoute.path !== '/auth/welcome'
	) {
		const redirect = resolvedRoute.path.startsWith('/auth/') ? null : resolvedRoute.fullPath

		await navigateTo(
			`/auth/welcome?authToken=${oauthCode}${
				redirect ? `&redirect=${encodeURIComponent(redirect)}` : ''
			}`,
		)
	}

	const tokenStr = normalizeAuthToken(authCookie.value)
	let shouldRefresh = false

	if (authCookie.value != null && tokenStr === '') {
		authCookie.value = null
	} else if (tokenStr) {
		auth.token = tokenStr

		if (!auth.token.startsWith('mra_')) {
			return auth
		}

		try {
			auth.user = (await useBaseFetch(
				'user',
				{
					apiVersion: 3,
					headers: {
						Authorization: auth.token,
					},
				},
				true,
			)) as Labrinth.Users.v2.User
		} catch (error) {
			// only refresh when the token was rejected. not on timeouts or other errors (think this was the cause of random logouts)
			shouldRefresh = isAuthFailure(error)
		}
	}

	if (!auth.user && auth.token && shouldRefresh) {
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
			)) as { session: unknown }

			auth.token = normalizeAuthToken(session.session)
			if (auth.token) {
				authCookie.value = auth.token
				try {
					auth.user = (await useBaseFetch(
						'user',
						{
							apiVersion: 3,
							headers: {
								Authorization: auth.token,
							},
						},
						true,
					)) as Labrinth.Users.v2.User
				} catch (error) {
					if (isAuthFailure(error)) {
						clearAuthCookie(auth, authCookie)
					}
				}
			} else {
				clearAuthCookie(auth, authCookie)
			}
		} catch (error) {
			if (isAuthFailure(error)) {
				clearAuthCookie(auth, authCookie)
			}
		}
	}

	if (auth.user && auth.token) {
		rememberStoredAccount(auth.user, auth.token)
	}

	return auth
}

export const getSignInRedirectPath = (route: FullPathRoute) => {
	const fullPath = route.fullPath
	if (fullPath === '/auth' || fullPath.startsWith('/auth/')) return undefined
	return fullPath
}

export const getSignInRouteObj = (route: FullPathRoute, redirectOverride?: string | null) => ({
	path: '/auth/sign-in',
	query: {
		redirect: redirectOverride ?? getSignInRedirectPath(route),
	},
})

export const ADD_ACCOUNT_QUERY_PARAM = 'add_account'

export const getAuthUrl = (provider: string, redirect?: string, includeAppSession = false) => {
	const config = useRuntimeConfig()
	const route = useNativeRoute()
	const launcher = getQueryString(route.query.launcher)
	const addingAccount =
		route.query[ADD_ACCOUNT_QUERY_PARAM] !== undefined || route.path === '/auth/reauthenticate'

	const callbackUrl = new URL('/auth/sign-in', config.public.siteUrl)
	if (launcher) {
		callbackUrl.searchParams.set('launcher', launcher)

		for (const param of LAUNCHER_AUTH_QUERY_PARAMS) {
			const value = getQueryString(route.query[param])
			if (value) {
				callbackUrl.searchParams.set(param, value)
			}
		}
		if (includeAppSession) {
			callbackUrl.searchParams.set(LAUNCHER_APP_SESSION_QUERY_PARAM, 'true')
		}
	} else if (redirect) {
		callbackUrl.searchParams.set('redirect', redirect)
	}

	if (addingAccount) {
		callbackUrl.searchParams.set(ADD_ACCOUNT_QUERY_PARAM, 'true')
	}

	return `${config.public.apiBaseUrl}auth/init?provider=${provider}&url=${encodeURIComponent(callbackUrl.toString())}`
}

export const promotePendingSignInOAuthProvider = () => {
	if (!import.meta.client) return
	const pending = useStorage<string | null>(
		'auth-pending-sign-in-oauth-provider',
		null,
		undefined,
		{ initOnMounted: true },
	)
	if (!pending.value) return
	const last = useStorage<string | null>('auth-last-sign-in-oauth-provider', null, undefined, {
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
