import dayjs from 'dayjs'

import {
	isOwnedByOthers,
	isSharedLibraryInstance,
	isUninstalledSharedInstance,
	type LibraryInstance,
} from '@/features/library/library-instance'
import {
	FAVORITES_GROUP_ID,
	type InstanceGroupDefinition,
} from '@/platform/app-lib/instance-groups/types'

import {
	type InstanceGroup,
	type LibraryFilters,
	type LibraryGroupBy,
	type LibraryInstanceType,
	type LibrarySort,
	type LibraryTab,
	NO_GROUPING_GROUP_ID,
	UNGROUPED_GROUP_ID,
} from './types'

const loaderPriority: Record<string, number> = {
	fabric: 0,
	forge: 1,
	neoforge: 2,
}

export function getLibraryInstanceType(
	instance: LibraryInstance,
	serverProjectIds: Set<string>,
): LibraryInstanceType {
	if (isUninstalledSharedInstance(instance)) return 'modpack'
	if (!instance.link) return 'custom'
	if (serverProjectIds.has(instance.link.project_id ?? '')) return 'server'
	return 'modpack'
}

export function matchesLibraryFilters(
	instance: LibraryInstance,
	filters: LibraryFilters,
	instanceType: LibraryInstanceType,
) {
	const matches = (selected: string[], value: string) =>
		selected.length === 0 || selected.includes(value)

	return (
		matches(filters.sharing, isSharedLibraryInstance(instance) ? 'shared' : 'not_shared') &&
		matches(filters.owner, isOwnedByOthers(instance) ? 'others' : 'me') &&
		matches(filters.instanceType, instanceType) &&
		matches(filters.gameVersion, instance.game_version) &&
		matches(filters.loader, instance.loader)
	)
}

const getTimestamp = (instance: LibraryInstance, key: 'last_played' | 'created' | 'modified') =>
	dayjs(isUninstalledSharedInstance(instance) ? 0 : (instance[key] ?? 0))

const getTimePlayed = (instance: LibraryInstance) =>
	isUninstalledSharedInstance(instance)
		? 0
		: instance.recent_time_played + instance.submitted_time_played

export function getOrderedLibraryGroupIds(
	groups: InstanceGroupDefinition[],
	storedUngroupedPosition: number,
) {
	const groupIds = groups
		.filter((group) => group.id !== FAVORITES_GROUP_ID)
		.map((group) => group.id)
	const ungroupedPosition = Math.min(
		Math.max(
			Number.isFinite(storedUngroupedPosition)
				? Math.trunc(storedUngroupedPosition)
				: Number.MAX_SAFE_INTEGER,
			0,
		),
		groupIds.length,
	)
	groupIds.splice(ungroupedPosition, 0, UNGROUPED_GROUP_ID)
	return groupIds
}

export type GroupLibraryInstancesOptions = {
	instances: LibraryInstance[]
	search: string
	sort: { field: LibrarySort; direction: 'asc' | 'desc' }
	groupBy: LibraryGroupBy
	tab: LibraryTab
	groups: InstanceGroupDefinition[]
	groupOrder: Map<string, number>
	getInstanceType: (instance: LibraryInstance) => LibraryInstanceType
	formatLoaderName: (loader: string) => string
	formatInstanceType: (instanceType: LibraryInstanceType) => string
}

export function groupLibraryInstances({
	instances,
	search,
	sort,
	groupBy,
	tab,
	groups,
	groupOrder,
	getInstanceType,
	formatLoaderName,
	formatInstanceType,
}: GroupLibraryInstancesOptions): InstanceGroup[] {
	const query = search.toLowerCase()
	const visibleInstances = instances.filter((instance) =>
		instance.name.toLowerCase().includes(query),
	)

	const compareInstances: Record<LibrarySort, (a: LibraryInstance, b: LibraryInstance) => number> =
		{
			Name: (a, b) => a.name.localeCompare(b.name, undefined, { numeric: true }),
			Loader: (a, b) => formatLoaderName(a.loader).localeCompare(formatLoaderName(b.loader)),
			'Game version': (a, b) =>
				a.game_version.localeCompare(b.game_version, undefined, { numeric: true }),
			'Last played': (a, b) => getTimestamp(a, 'last_played').diff(getTimestamp(b, 'last_played')),
			'Hours played': (a, b) => getTimePlayed(a) - getTimePlayed(b),
			'Date created': (a, b) => getTimestamp(a, 'created').diff(getTimestamp(b, 'created')),
			'Date modified': (a, b) => getTimestamp(a, 'modified').diff(getTimestamp(b, 'modified')),
		}
	const compare = compareInstances[sort.field]
	const sign = sort.direction === 'asc' ? 1 : -1
	visibleInstances.sort((a, b) => sign * compare(a, b))

	const groupedInstances = new Map<string, { name: string; instances: LibraryInstance[] }>()
	const addToGroup = (id: string, name: string, instance?: LibraryInstance) => {
		const group = groupedInstances.get(id) ?? { name, instances: [] }
		if (instance) group.instances.push(instance)
		groupedInstances.set(id, group)
	}
	const groupsById = new Map(groups.map((group) => [group.id, group]))

	for (const instance of visibleInstances) {
		switch (groupBy) {
			case 'Instance type': {
				const instanceType = getInstanceType(instance)
				addToGroup(`Instance type:${instanceType}`, formatInstanceType(instanceType), instance)
				break
			}
			case 'Loader': {
				const name = formatLoaderName(instance.loader)
				addToGroup(`Loader:${name}`, name, instance)
				break
			}
			case 'Game version':
				addToGroup(`Game version:${instance.game_version}`, instance.game_version, instance)
				break
			case 'Group': {
				const instanceGroupIds = isUninstalledSharedInstance(instance) ? [] : instance.group_ids
				if (instanceGroupIds.length === 0) {
					addToGroup(UNGROUPED_GROUP_ID, '', instance)
				}
				for (const groupId of instanceGroupIds) {
					addToGroup(groupId, groupsById.get(groupId)?.name ?? groupId, instance)
				}
				break
			}
			case 'None':
				addToGroup(NO_GROUPING_GROUP_ID, '', instance)
				break
		}
	}

	if (groupBy === 'Group' && tab === 'installed') {
		addToGroup(UNGROUPED_GROUP_ID, '')
		for (const group of groups) {
			addToGroup(group.id, group.name)
		}
	}

	const result = Array.from(groupedInstances, ([id, group]) => ({
		id,
		key: group.name,
		instances: group.instances,
	}))
	const byKey = (a: InstanceGroup, b: InstanceGroup) =>
		a.key.localeCompare(b.key) || a.id.localeCompare(b.id)

	if (sort.field === 'Name') result.sort(byKey)

	switch (groupBy) {
		case 'Loader':
			result.sort((a, b) => {
				const aPriority = loaderPriority[a.key.toLowerCase()] ?? Number.MAX_SAFE_INTEGER
				const bPriority = loaderPriority[b.key.toLowerCase()] ?? Number.MAX_SAFE_INTEGER
				return aPriority - bPriority || byKey(a, b)
			})
			break
		case 'Instance type':
			result.sort(byKey)
			break
		case 'Game version':
			result.sort((a, b) => b.key.localeCompare(a.key, undefined, { numeric: true }))
			break
		case 'Group':
			result.sort((a, b) => {
				if (a.id === b.id) return 0
				if (a.id === FAVORITES_GROUP_ID) return -1
				if (b.id === FAVORITES_GROUP_ID) return 1
				const aOrder = groupOrder.get(a.id) ?? Number.MAX_SAFE_INTEGER
				const bOrder = groupOrder.get(b.id) ?? Number.MAX_SAFE_INTEGER
				return aOrder - bOrder || byKey(a, b)
			})
			break
	}

	return result
}
