import type { SortDirection, SortKind } from '@modrinth/ui'

import type { LibraryInstance } from '@/features/library/library-instance'

export const UNGROUPED_GROUP_ID = 'group:none'
export const NO_GROUPING_GROUP_ID = 'None:None'

export const librarySortOptions = [
	'Name',
	'Last played',
	'Hours played',
	'Date created',
	'Date modified',
	'Loader',
	'Game version',
] as const

export const libraryGroupOptions = [
	'Group',
	'Instance type',
	'Loader',
	'Game version',
	'None',
] as const

export const librarySortKinds: Record<LibrarySort, SortKind> = {
	Name: 'alphabetical',
	'Last played': 'date',
	'Hours played': 'numeric',
	'Date created': 'date',
	'Date modified': 'date',
	Loader: 'alphabetical',
	'Game version': 'numeric',
}

export const getDefaultSortDirection = (kind: SortKind): SortDirection =>
	kind === 'alphabetical' ? 'asc' : 'desc'

export type LibraryTab = 'installed' | 'available'
export type LibrarySort = (typeof librarySortOptions)[number]
export type LibraryGroupBy = (typeof libraryGroupOptions)[number]
export type LibraryInstanceType = 'custom' | 'modpack' | 'server'
export type LibraryFilters = Record<
	'instanceType' | 'gameVersion' | 'loader' | 'sharing' | 'owner',
	string[]
>

export const emptyLibraryFilters = (): LibraryFilters => ({
	instanceType: [],
	gameVersion: [],
	loader: [],
	sharing: [],
	owner: [],
})

export type LibraryDisplayState = {
	group: LibraryGroupBy
	sortBy: LibrarySort
	sortDirection?: SortDirection
	collapsedGroups: string[]
	ungroupedGroupPosition: number
}

export type InstanceGroup = {
	id: string
	key: string
	instances: LibraryInstance[]
}

export type LibraryInstanceSelection = {
	instanceId: string
	groupId: string
}

export type ActiveInstanceGroupDrag = {
	instances: LibraryInstanceSelection[]
	primaryInstanceId: string
	fromGroup: string | null
}

export const getLibraryInstanceSelectionKey = ({ instanceId, groupId }: LibraryInstanceSelection) =>
	JSON.stringify([groupId, instanceId])
