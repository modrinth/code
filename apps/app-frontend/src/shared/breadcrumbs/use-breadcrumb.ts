import { toValue, watch } from 'vue'

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
