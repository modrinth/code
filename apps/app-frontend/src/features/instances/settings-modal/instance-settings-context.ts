import { createContext } from '@modrinth/ui'
import type { ComputedRef, Ref } from 'vue'

import type { GameInstance } from '@/helpers/types'
import type { UnsavedChangesController } from '@/shared/unsaved-changes/types'

export interface InstanceSettingsContext {
	instance: ComputedRef<GameInstance>
	offline?: boolean
	isMinecraftServer: Ref<boolean>
	onUnlinked: () => void
	closeModal?: (onAfterClose?: () => void) => void
	registerUnsavedChangesController: (controller: UnsavedChangesController | null) => void
}

export const [injectInstanceSettings, provideInstanceSettings] =
	createContext<InstanceSettingsContext>('InstanceSettingsModal', 'instanceSettings')
