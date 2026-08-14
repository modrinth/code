import type { Labrinth } from '@modrinth/api-client'
import { createContext } from '@modrinth/ui'
import type { ComputedRef, Ref } from 'vue'

import type { GameInstance } from '@/helpers/types'

export interface InstancePageContext {
	readonly instanceId: ComputedRef<string>
	readonly instance: ComputedRef<GameInstance>
	readonly linkedProject: ComputedRef<Labrinth.Projects.v3.Project | undefined>
	readonly isServerInstance: ComputedRef<boolean>
	readonly sharedInstanceUpdateAvailable: ComputedRef<boolean>
	readonly offline: Readonly<Ref<boolean>>
	readonly playing: ComputedRef<boolean>
	readonly loading: Readonly<Ref<boolean>>
	readonly stopping: Readonly<Ref<boolean>>
	refreshInstance: () => Promise<void>
	refreshPlayState: () => Promise<void>
	play: (source: string) => Promise<void>
	stop: (source: string) => Promise<void>
	playServer: () => Promise<void>
	openSettings: (tab?: number) => void
	browseContent: (projectType?: string) => Promise<void>
	browseServers: () => Promise<void>
	reviewSharedInstanceUpdate: (event?: MouseEvent) => void
}

export const [injectInstancePage, provideInstancePage] =
	createContext<InstancePageContext>('InstancePage')
