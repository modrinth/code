import { createContext } from '.'
import type { ModrinthServer } from '@modrinth/ui'
import type { Ref } from 'vue'

export interface ServerContext {
  server: Ref<ModrinthServer>
}

export const [injectModrinthServerContext, provideModrinthServerContext] =
  createContext<ServerContext>('root', 'serverContext')
