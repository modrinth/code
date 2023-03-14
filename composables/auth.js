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
    headers: {},
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

  if (route.query.code) {
    authCookie.value = route.query.code
  }

  if (authCookie.value) {
    auth.token = authCookie.value
    try {
      auth.user = await useBaseFetch('user', {
        headers: {
          Authorization: auth.token,
        },
      })
    } catch {}

    auth.headers = {
      headers: {
        Authorization: auth.token,
      },
    }
  }

  return auth
}

export const getAuthUrl = () => {
  const config = useRuntimeConfig()
  const route = useRoute()

  return `${config.public.apiBaseUrl}auth/init?url=${config.public.siteUrl}${route.fullPath}`
}
