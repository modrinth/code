import { ofetch } from 'ofetch'

/**
 * This will be our useFetch to wrap Http requests to Labrinth or other services.
 * We will configure headers, etc. in this one place rather.
 * Error-handling will still need handled at the page level currently.
 */
export default async (url) => await ofetch(url)
