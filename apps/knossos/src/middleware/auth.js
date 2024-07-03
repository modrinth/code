const whitelistedParams = ['flow', 'error']

export default defineNuxtRouteMiddleware(async (_to, from) => {
  const config = useRuntimeConfig()
  const auth = await useAuth()

  if (!auth.value.user) {
    const fullPath = from.fullPath

    const url = new URL(fullPath, config.public.apiBaseUrl)

    const extractedParams = whitelistedParams.reduce((acc, param) => {
      if (url.searchParams.has(param)) {
        acc[param] = url.searchParams.get(param)
        url.searchParams.delete(param)
      }
      return acc
    }, {})

    const redirectPath = encodeURIComponent(url.pathname + url.search)

    return await navigateTo(
      {
        path: '/auth/sign-in',
        query: {
          redirect: redirectPath,
          ...extractedParams,
        },
      },
      {
        replace: true,
      }
    )
  }
})
