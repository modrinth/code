import { createContext } from './create-context'

export interface IconCacheContext {
	cacheIcon: (source: string | Blob) => Promise<string>
	peekIcon?: (source: string | Blob) => string | undefined
}

export const [injectIconCache, provideIconCache] = createContext<IconCacheContext>('IconCache')
