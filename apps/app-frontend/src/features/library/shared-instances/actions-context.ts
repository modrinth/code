import { createContext } from '@modrinth/ui'
import type { ComputedRef } from 'vue'

import type { SharedInstanceInstallOptions, UninstalledSharedInstance } from './types'

export interface SharedInstanceActionsContext {
	loadingIds: ComputedRef<Set<string>>
	removing: ComputedRef<boolean>
	install: (instance: UninstalledSharedInstance, options?: SharedInstanceInstallOptions) => void
	prefetchInstall: (instance: UninstalledSharedInstance) => void
	requestRemove: (instances: UninstalledSharedInstance[], event?: MouseEvent) => void
}

export const [injectSharedInstanceActions, provideSharedInstanceActions] =
	createContext<SharedInstanceActionsContext>('SharedInstanceModals', 'sharedInstanceActions')
