import { $fetch, FetchError } from 'ofetch'

interface PyroFetchOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'
  body?: Record<string, unknown>
  accept?: 'application/json' | (string & {})
  version?: number
  session?: string
  override?: {
    url: string
    token: string
  }
}

export class PyroFetchError extends Error {
  constructor(
    message: string,
    public statusCode?: number,
    public originalError?: Error,
  ) {
    super(message)
    this.name = 'PyroFetchError'
  }
}

export async function usePyroFetch<T>(path: string, options: PyroFetchOptions = {}): Promise<T> {
  const authToken = options.session

  if (!authToken) {
    throw new PyroFetchError('Cannot pyrofetch without auth', 10000)
  }

  const { method = 'GET', body, accept = 'application/json', version = 0, override } = options

  const base = 'https://archon.pyro.host'

  const fullUrl = override?.url
    ? `https://${override.url}/${path.replace(/^\//, '')}`
    : `${base}/modrinth/v${version}/${path.replace(/^\//, '')}`

  type HeadersRecord = Record<string, string>

  const headers: HeadersRecord = {
    Accept: accept,
    Authorization: `Bearer ${override?.token ?? authToken}`,
    'Access-Control-Allow-Headers': 'Authorization',
    'User-Agent': 'Pyro/1.0 (https://pyro.host)',
    Vary: 'Accept, Origin',
  }

  if (typeof window !== 'undefined') {
    headers.Origin = window.location.origin
  }

  if (['POST', 'PUT', 'PATCH', 'DELETE'].includes(method) && body) {
    headers['Content-Type'] = 'application/json'
  }

  try {
    const response = await $fetch<T>(fullUrl, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
      timeout: 10000,
      retry: method === 'GET' ? 3 : 0,
    })
    return response
  } catch (error) {
    console.error('Fetch error:', error)
    if (error instanceof FetchError) {
      const statusCode = error.response?.status
      const statusText = error.response?.statusText || 'Unknown error'
      const errorMessages: { [key: number]: string } = {
        400: 'Bad Request',
        401: 'Unauthorized',
        403: 'Forbidden',
        404: 'Not Found',
        500: 'Internal Server Error',
      }
      const message =
        statusCode && statusCode in errorMessages
          ? errorMessages[statusCode]
          : `HTTP Error: ${statusCode || 'unknown'} ${statusText}`
      throw new PyroFetchError(`[PYRO] ${message}`, statusCode, error)
    }
    throw new PyroFetchError(
      '[PYRO] An unexpected error occurred during the fetch operation.',
      undefined,
      error as Error,
    )
  }
}
