import { createContext } from './create-context'

export type ServerPlayTarget = {
	serverId: string
	worldId: string
}

export const [injectServerPlay, provideServerPlay] = createContext<{
	play: (target: ServerPlayTarget) => Promise<void>
}>('root', 'serverPlay')
