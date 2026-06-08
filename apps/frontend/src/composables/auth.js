function normalizeAuthToken(value) {
	if (typeof value === 'string') {
		return value
	}
	return ''
}

export const useAuth = async (oldToken = null) => {
	const auth = useState('auth', () => ({
		user: null,
		token: '',
		headers: {},
	}))

	if (!auth.value.user || oldToken) {
		auth.value = await initAuth(oldToken)
	}

	return auth
}

export const initAuth = async (oldToken = null) => {
	const auth = {
		user: null,
		token: '',
	}

	if (oldToken === 'none') {
		return auth
	}

	const route = useRoute()
	const config = useRuntimeConfig()
	const authCookie = useCookie('auth-token', {
		maxAge: 60 * 60 * 24 * 365 * 10,
		sameSite: 'lax',
		secure: config.public.cookieSecure,
		httpOnly: false,
		path: '/',
	})

	if (oldToken) {
		const normalized = normalizeAuthToken(oldToken)
		if (normalized) {
			authCookie.value = normalized
		}
	}

	const oauthCode = normalizeAuthToken(route.query.code)
	if (oauthCode && !route.fullPath.includes('new_account=true')) {
		authCookie.value = oauthCode
	}

	if (route.fullPath.includes('new_account=true') && route.path !== '/auth/welcome') {
		const redirect = route.path.startsWith('/auth/') ? null : route.fullPath

		await navigateTo(
			`/auth/welcome?authToken=${oauthCode}${
				redirect ? `&redirect=${encodeURIComponent(redirect)}` : ''
			}`,
		)
	}

	const tokenStr = normalizeAuthToken(authCookie.value)

	if (authCookie.value != null && tokenStr === '') {
		authCookie.value = null
	} else if (tokenStr) {
		auth.token = tokenStr

		if (!auth.token.startsWith('mra_')) {
			return auth
		}

		try {
			auth.user = await useBaseFetch(
				'user',
				{
					apiVersion: 3,
					headers: {
						Authorization: auth.token,
					},
				},
				true,
			)
		} catch {
			/* empty */
		}
	}

	if (!auth.user && auth.token && typeof auth.token === 'string') {
		try {
			const session = await useBaseFetch(
				'session/refresh',
				{
					method: 'POST',
					headers: {
						Authorization: auth.token,
					},
				},
				true,
			)

			auth.token = normalizeAuthToken(session.session)
			if (auth.token) {
				authCookie.value = auth.token
				auth.user = await useBaseFetch(
					'user',
					{
						apiVersion: 3,
						headers: {
							Authorization: auth.token,
						},
					},
					true,
				)
			} else {
				authCookie.value = null
				auth.token = ''
			}
		} catch {
			authCookie.value = null
		}
	}

	return auth
}

export const getSignInRedirectPath = (route) => {
	const fullPath = route.fullPath
	if (fullPath === '/auth' || fullPath.startsWith('/auth/')) {
		return '/dashboard'
	}
	return fullPath
}

export const getSignInRouteObj = (route, redirectOverride) => ({
	path: '/auth/sign-in',
	query: {
		redirect: redirectOverride ?? getSignInRedirectPath(route),
	},
})

export const getAuthUrl = (provider, redirect = '/dashboard') => {
	const config = useRuntimeConfig()
	const route = useNativeRoute()

	const fullURL = route.query.launcher
		? getLauncherRedirectUrl(route)
		: `${config.public.siteUrl}/auth/sign-in?redirect=${encodeURIComponent(redirect)}`

	return `${config.public.apiBaseUrl}auth/init?provider=${provider}&url=${encodeURIComponent(fullURL)}`
}

export const removeAuthProvider = async (provider) => {
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

export const getLauncherRedirectUrl = (route) => {
	const usesLocalhostRedirectionScheme =
		['4', '6'].includes(route.query.ipver) && Number(route.query.port) < 65536

	return usesLocalhostRedirectionScheme
		? `http://${route.query.ipver === '4' ? '127.0.0.1' : '[::1]'}:${route.query.port}`
		: `https://launcher-files.modrinth.com`
}
