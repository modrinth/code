import { computed, useTemplateRef } from 'vue'

import { provideSharedInstanceActions } from './actions-context'
import type { SharedInstanceModalsExposed } from './types'

export function setupSharedInstanceActionsProvider() {
	const modals = useTemplateRef<SharedInstanceModalsExposed>('sharedInstanceModals')

	provideSharedInstanceActions({
		loadingIds: computed(() => modals.value?.loadingIds ?? new Set()),
		removing: computed(() => modals.value?.removing ?? false),
		install: (instance, options) => void modals.value?.install(instance, options),
		prefetchInstall: (instance) => modals.value?.prefetchInstall(instance),
		requestRemove: (instances, event) => modals.value?.requestRemove(instances, event),
	})
}
