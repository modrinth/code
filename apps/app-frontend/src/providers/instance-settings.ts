import { createContext } from '@modrinth/ui'
import type { ComputedRef, Ref } from 'vue'

import type { GameInstance } from '@/helpers/types'

export interface InstanceSettingsContext {
	instance: ComputedRef<GameInstance>
	offline?: boolean
	isMinecraftServer: Ref<boolean>
	onUnlinked: () => void
}

export const [injectInstanceSettings, provideInstanceSettings] =
	createContext<InstanceSettingsContext>('InstanceSettingsModal', 'instanceSettings')
