import { createContext } from '@modrinth/ui'
import { computed, type ComputedRef, useTemplateRef } from 'vue'

import type SharedInstanceModals from './shared-instance-modals.vue'
import type { UninstalledSharedInstance } from './types'

export type SharedInstanceInstallOptions = {
	event?: MouseEvent
	onInstalled?: () => void
}

export interface SharedInstanceActionsContext {
	loadingIds: ComputedRef<Set<string>>
	removing: ComputedRef<boolean>
	install: (instance: UninstalledSharedInstance, options?: SharedInstanceInstallOptions) => void
	prefetchInstall: (instance: UninstalledSharedInstance) => void
	requestRemove: (instances: UninstalledSharedInstance[], event?: MouseEvent) => void
}

export const [injectSharedInstanceActions, provideSharedInstanceActions] =
	createContext<SharedInstanceActionsContext>('SharedInstanceModals', 'sharedInstanceActions')

export function setupSharedInstanceActionsProvider() {
	const modals = useTemplateRef<InstanceType<typeof SharedInstanceModals>>('sharedInstanceModals')

	provideSharedInstanceActions({
		loadingIds: computed(() => modals.value?.loadingIds ?? new Set()),
		removing: computed(() => modals.value?.removing ?? false),
		install: (instance, options) => void modals.value?.install(instance, options),
		prefetchInstall: (instance) => modals.value?.prefetchInstall(instance),
		requestRemove: (instances, event) => modals.value?.requestRemove(instances, event),
	})
}
