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
  const authCookie = useCookie('auth-token', {
    maxAge: 60 * 60 * 24 * 365 * 10,
    sameSite: 'lax',
    secure: true,
    httpOnly: false,
    path: '/',
  })

  if (oldToken) {
    authCookie.value = oldToken
  }

  if (route.query.code && !route.fullPath.includes('new_account=true')) {
    authCookie.value = route.query.code
  }

  if (authCookie.value) {
    auth.token = authCookie.value

    if (!auth.token || !auth.token.startsWith('mra_')) {
      return auth
    }

    try {
      auth.user = await useBaseFetch(
        'user',
        {
          headers: {
            Authorization: auth.token,
          },
        },
        true
      )
    } catch {}
  }

  if (!auth.user && auth.token) {
    try {
      const session = await useBaseFetch(
        'session/refresh',
        {
          method: 'POST',
          headers: {
            Authorization: auth.token,
          },
        },
        true
      )

      auth.token = session.session
      authCookie.value = auth.token

      auth.user = await useBaseFetch(
        'user',
        {
          headers: {
            Authorization: auth.token,
          },
        },
        true
      )
    } catch {
      authCookie.value = null
    }
  }

  return auth
}

export const getAuthUrl = (provider, redirect = '') => {
  const config = useRuntimeConfig()
  const route = useNativeRoute()

  if (redirect === '') {
    redirect = route.path
  }
  const fullURL = `${config.public.siteUrl}${redirect}`

  return `${config.public.apiBaseUrl}auth/init?url=${fullURL}&provider=${provider}`
}

export const removeAuthProvider = async (provider) => {
  startLoading()
  try {
    const auth = await useAuth()

    await useBaseFetch('auth/provider', {
      method: 'DELETE',
      body: {
        provider,
      },
    })
    await useAuth(auth.value.token)
  } catch (err) {
    const data = useNuxtApp()
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })
  }
  stopLoading()
}
