import { createContext } from './create-context'

export interface IconCacheContext {
	cacheIcon: (source: string | Blob) => Promise<string>
}

export const [injectIconCache, provideIconCache] = createContext<IconCacheContext>('IconCache')
