export const useBaseFetch = async (url, options = {}, skipAuth = false) => {
  const config = useRuntimeConfig()
  const base = process.server ? config.apiBaseUrl : config.public.apiBaseUrl

  if (!options.headers) {
    options.headers = {}
  }

  if (process.server) {
    options.headers['x-ratelimit-key'] = config.rateLimitKey
  }

  if (!skipAuth) {
    const auth = await useAuth()

    options.headers.Authorization = auth.value.token
  }

  return await $fetch(`${base}${url}`, options)
}
