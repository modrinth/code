import { createContext } from '@modrinth/ui'
import type { Ref } from 'vue'

import type { GameInstance } from '@/helpers/types'

export interface InstanceSettingsContext {
	instance: GameInstance
	offline?: boolean
	isMinecraftServer: Ref<boolean>
	onUnlinked: () => void
}

export const [injectInstanceSettings, provideInstanceSettings] =
	createContext<InstanceSettingsContext>('InstanceSettingsModal', 'instanceSettings')
