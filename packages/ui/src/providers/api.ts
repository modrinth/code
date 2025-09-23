import type { ModrinthApi } from '@modrinth/utils'

import { createContext } from '.'

export const [injectApi, provideApi] = createContext<ModrinthApi>('root', 'apiContext')
