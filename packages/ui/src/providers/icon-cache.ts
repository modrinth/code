import { createContext } from './create-context'

export interface IconCacheContext {
	cacheIcon: (source: string) => Promise<string>
	peekIcon?: (source: string) => string | undefined
}

export const [injectIconCache, provideIconCache] = createContext<IconCacheContext>('IconCache')
