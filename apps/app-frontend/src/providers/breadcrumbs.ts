import { createContext } from '@modrinth/ui'
import {
	type Component,
	computed,
	type ComputedRef,
	type MaybeRefOrGetter,
	shallowRef,
	toValue,
	watch,
} from 'vue'
import type { RouteLocationRaw } from 'vue-router'

export type BreadcrumbVisual =
	| {
			type: 'icon'
			component: Component
	  }
	| {
			type: 'image'
			src?: string | null
			alt?: string
			circle?: boolean
			tintBy?: string | null
	  }

export interface BreadcrumbDefinition {
	slot: string
	id: MaybeRefOrGetter<string>
	label: MaybeRefOrGetter<string>
	to?: MaybeRefOrGetter<RouteLocationRaw | undefined>
	visual?: MaybeRefOrGetter<BreadcrumbVisual | undefined>
}

export interface ResolvedBreadcrumb {
	slot: string
	id: string
	label: string
	to?: RouteLocationRaw
	visual?: BreadcrumbVisual
}

export interface BreadcrumbHandle {
	readonly slot: string
	activate: () => void
	reset: () => void
	pop: () => void
}

export interface BreadcrumbManager {
	readonly entries: ComputedRef<ResolvedBreadcrumb[]>
	reset: (definition: BreadcrumbDefinition) => BreadcrumbHandle
	push: (
		definition: BreadcrumbDefinition,
		options?: { parent?: BreadcrumbHandle },
	) => BreadcrumbHandle
	find: (slot: string) => BreadcrumbHandle | undefined
}

interface InternalBreadcrumb {
	token: symbol
	definition: BreadcrumbDefinition
	parentToken?: symbol
	parentSlot?: string
	root: boolean
	handle: BreadcrumbHandle
}

const [injectBreadcrumbManager, provideBreadcrumbManager] = createContext<BreadcrumbManager>(
	'root',
	'breadcrumbManager',
)
const [injectBreadcrumbParent, provideBreadcrumbParent] = createContext<BreadcrumbHandle>(
	'BreadcrumbParent',
	'breadcrumbParent',
)

export { injectBreadcrumbManager, provideBreadcrumbManager, provideBreadcrumbParent }

export function createBreadcrumbManager(): BreadcrumbManager {
	const stack = shallowRef<InternalBreadcrumb[]>([])

	function findIndex(entry: InternalBreadcrumb): number {
		const tokenIndex = stack.value.findIndex((candidate) => candidate.token === entry.token)
		if (tokenIndex !== -1) return tokenIndex
		return stack.value.findIndex((candidate) => candidate.definition.slot === entry.definition.slot)
	}

	function findParentIndex(entry: InternalBreadcrumb): number {
		if (entry.parentToken) {
			const tokenIndex = stack.value.findIndex((candidate) => candidate.token === entry.parentToken)
			if (tokenIndex !== -1) return tokenIndex
		}
		if (entry.parentSlot) {
			return stack.value.findIndex((candidate) => candidate.definition.slot === entry.parentSlot)
		}
		return stack.value.length - 1
	}

	function activate(entry: InternalBreadcrumb) {
		if (entry.root) {
			stack.value = [entry]
			return
		}

		const existingIndex = findIndex(entry)
		if (existingIndex !== -1) {
			stack.value = [...stack.value.slice(0, existingIndex), entry]
			return
		}

		const parentIndex = findParentIndex(entry)
		stack.value = [...stack.value.slice(0, parentIndex + 1), entry]
	}

	function resetTo(entry: InternalBreadcrumb) {
		stack.value = [entry]
	}

	function pop(entry: InternalBreadcrumb) {
		const index = stack.value.findIndex((candidate) => candidate.token === entry.token)
		if (index !== -1) {
			stack.value = stack.value.slice(0, index)
		}
	}

	function createHandle(
		definition: BreadcrumbDefinition,
		options: { parent?: BreadcrumbHandle; root: boolean },
	): BreadcrumbHandle {
		const entry = {} as InternalBreadcrumb
		const handle: BreadcrumbHandle = {
			slot: definition.slot,
			activate: () => activate(entry),
			reset: () => resetTo(entry),
			pop: () => pop(entry),
		}

		Object.assign(entry, {
			token: Symbol(definition.slot),
			definition,
			parentToken: options.parent
				? stack.value.find((candidate) => candidate.handle === options.parent)?.token
				: undefined,
			parentSlot: options.parent?.slot,
			root: options.root,
			handle,
		})

		return handle
	}

	function reset(definition: BreadcrumbDefinition): BreadcrumbHandle {
		const handle = createHandle(definition, { root: true })
		handle.reset()
		return handle
	}

	function push(
		definition: BreadcrumbDefinition,
		options: { parent?: BreadcrumbHandle } = {},
	): BreadcrumbHandle {
		const handle = createHandle(definition, { ...options, root: false })
		handle.activate()
		return handle
	}

	const entries = computed<ResolvedBreadcrumb[]>(() =>
		stack.value.map(({ definition }) => ({
			slot: definition.slot,
			id: toValue(definition.id),
			label: toValue(definition.label),
			to: definition.to === undefined ? undefined : toValue(definition.to),
			visual: definition.visual === undefined ? undefined : toValue(definition.visual),
		})),
	)

	return {
		entries,
		reset,
		push,
		find: (slot) => stack.value.find((entry) => entry.definition.slot === slot)?.handle,
	}
}

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
