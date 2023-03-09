export const useBaseFetch = async (url, options = {}) => {
  const config = useRuntimeConfig()
  const base = process.server ? config.apiBaseUrl : config.public.apiBaseUrl

  if (options.headers && process.server) {
    options.headers['x-ratelimit-key'] = config.rateLimitKey
  } else if (process.server) {
    options.headers = {
      'x-ratelimit-key': config.rateLimitKey,
    }
  }

  return await $fetch(`${base}${url}`, options)
}
