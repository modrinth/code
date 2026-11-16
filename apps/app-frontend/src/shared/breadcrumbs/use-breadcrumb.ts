import { computed, type ComputedRef, toValue, watch } from 'vue'

import {
	type BreadcrumbDefinition,
	type BreadcrumbHandle,
	injectBreadcrumbManager,
	injectBreadcrumbParent,
} from './context'

function watchBreadcrumbIdentity(definition: BreadcrumbDefinition, handle: BreadcrumbHandle) {
	watch(
		() => toValue(definition.id),
		() => handle.activate(),
		{ flush: 'sync' },
	)
}

export function useRootBreadcrumb(definition: BreadcrumbDefinition): BreadcrumbHandle {
	const manager = injectBreadcrumbManager()
	const handle = manager.reset(definition)
	watchBreadcrumbIdentity(definition, handle)
	return handle
}

export function useBreadcrumb(
	definition: BreadcrumbDefinition,
	options: { parent?: BreadcrumbHandle } = {},
): BreadcrumbHandle {
	const manager = injectBreadcrumbManager()
	const parent = options.parent ?? injectBreadcrumbParent(null) ?? undefined
	const handle = manager.push(definition, { parent })
	watchBreadcrumbIdentity(definition, handle)
	return handle
}

/**
 * The ID of the hosting server in the current breadcrumb trail, if the user navigated here from a
 * server panel.
 */
export function useBreadcrumbServerId(): ComputedRef<string | null> {
	const manager = injectBreadcrumbManager()
	return computed(() => {
		const entry = manager.entries.value.find((candidate) => candidate.slot === 'server')
		const serverId = entry?.id.startsWith('server:') ? entry.id.slice('server:'.length) : ''
		return serverId || null
	})
}
