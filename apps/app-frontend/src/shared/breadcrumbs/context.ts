import { createContext } from '@modrinth/ui'
import type { Component, ComputedRef, MaybeRefOrGetter } from 'vue'
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

export const [injectBreadcrumbManager, provideBreadcrumbManager] = createContext<BreadcrumbManager>(
	'root',
	'breadcrumbManager',
)
export const [injectBreadcrumbParent, provideBreadcrumbParent] = createContext<BreadcrumbHandle>(
	'BreadcrumbParent',
	'breadcrumbParent',
)
