import { formatLoader, type SortMode, useVIntl } from '@modrinth/ui'
import { keepPreviousData, useQuery } from '@tanstack/vue-query'
import { useSessionStorage, useStorage } from '@vueuse/core'
import { computed, type MaybeRefOrGetter, ref, toValue, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { instanceGroupListQueryOptions } from '@/features/instances/groups/queries'
import { useHiddenInstances } from '@/features/library/hidden-instances/use-hidden-instances'
import {
	isUninstalledSharedInstance,
	type LibraryInstance,
	OWNED_SHARED_INSTANCES_FILTER,
} from '@/features/library/library-instance'
import { libraryInstanceTypeMessages } from '@/features/library/messages'
import type { UninstalledSharedInstance } from '@/features/library/shared-instances/types'
import { useSharedInstanceOwners } from '@/features/library/shared-instances/use-shared-instance-owners'
import type { GameInstance } from '@/helpers/types'
import { FAVORITES_GROUP_ID } from '@/platform/app-lib/instance-groups/types'

import {
	getLibraryInstanceType,
	getOrderedLibraryGroupIds,
	groupLibraryInstances,
	matchesLibraryFilters,
} from '../group-library-instances'
import { libraryServerProjectIdsQueryOptions } from '../queries'
import {
	emptyLibraryFilters,
	getDefaultSortDirection,
	type LibraryDisplayState,
	type LibraryFilters,
	libraryGroupOptions,
	type LibrarySort,
	librarySortKinds,
	librarySortOptions,
	type LibraryTab,
	UNGROUPED_GROUP_ID,
} from '../types'

export function useLibraryView({
	instances,
	sharedInstances,
}: {
	instances: MaybeRefOrGetter<GameInstance[]>
	sharedInstances: MaybeRefOrGetter<UninstalledSharedInstance[]>
}) {
	const { formatMessage } = useVIntl()
	const route = useRoute()
	const router = useRouter()
	const { hiddenIds } = useHiddenInstances()

	const installedInstances = computed(() => toValue(instances))
	const search = useSessionStorage('library-search', '')
	const activeTab = useSessionStorage<LibraryTab>('library-tab', 'installed')

	const storedFilters = useStorage<LibraryFilters>(
		'Instances-grid-filters',
		emptyLibraryFilters(),
		localStorage,
		{ mergeDefaults: true },
	)
	const presetFilters = ref<LibraryFilters | null>(null)
	const filters = computed<LibraryFilters>({
		get: () => presetFilters.value ?? storedFilters.value,
		set: (value) => {
			if (presetFilters.value) {
				presetFilters.value = value
			} else {
				storedFilters.value = value
			}
		},
	})

	watch(
		() => route.query.filter,
		(filter) => {
			if (filter !== OWNED_SHARED_INSTANCES_FILTER) return
			presetFilters.value = { ...emptyLibraryFilters(), sharing: ['shared'], owner: ['me'] }
			const { filter: _filter, ...query } = route.query
			void router.replace({ query })
		},
		{ immediate: true },
	)

	const displayState = useStorage<LibraryDisplayState>(
		'Instances-grid-display-state',
		{
			group: 'Group',
			sortBy: 'Last played',
			collapsedGroups: [],
			ungroupedGroupPosition: Number.MAX_SAFE_INTEGER,
		},
		localStorage,
		{ mergeDefaults: true },
	)

	if (!librarySortOptions.includes(displayState.value.sortBy)) {
		displayState.value.sortBy = 'Last played'
	}
	if (!libraryGroupOptions.includes(displayState.value.group)) {
		displayState.value.group = 'Group'
	}

	const sortMode = computed<SortMode<LibrarySort>>({
		get: () => ({
			field: displayState.value.sortBy,
			direction:
				displayState.value.sortDirection ??
				getDefaultSortDirection(librarySortKinds[displayState.value.sortBy]),
		}),
		set: ({ field, direction }) => {
			displayState.value.sortBy = field
			displayState.value.sortDirection = direction
		},
	})

	const isSearching = computed(() => search.value.length > 0)
	const hasActiveFilters = computed(() =>
		Object.values(filters.value).some((selectedValues) => selectedValues.length > 0),
	)
	const collapsedSectionKeys = computed(() => new Set(displayState.value.collapsedGroups))
	const getSectionKey = (sectionId: string) => `${displayState.value.group}:${sectionId}`

	const isSectionCollapsed = (sectionId: string) =>
		!isSearching.value && collapsedSectionKeys.value.has(getSectionKey(sectionId))

	const setSectionCollapsed = (sectionId: string, collapsed: boolean) => {
		if (isSearching.value) return

		const sectionKey = getSectionKey(sectionId)
		const collapsedSections = new Set(displayState.value.collapsedGroups)
		if (collapsed) {
			collapsedSections.add(sectionKey)
		} else {
			collapsedSections.delete(sectionKey)
		}
		displayState.value.collapsedGroups = [...collapsedSections]
	}

	const groupsQuery = useQuery(instanceGroupListQueryOptions())
	const groups = computed(() => groupsQuery.data.value ?? [])
	const groupsLoaded = computed(() => !groupsQuery.isPending.value)
	const orderedGroupIds = computed(() =>
		getOrderedLibraryGroupIds(groups.value, displayState.value.ungroupedGroupPosition),
	)
	const groupOrder = computed(
		() => new Map(orderedGroupIds.value.map((groupId, index) => [groupId, index])),
	)

	const linkedProjectIds = computed(() =>
		[
			...new Set(
				installedInstances.value.flatMap((instance) =>
					instance.link?.project_id ? [instance.link.project_id] : [],
				),
			),
		].sort(),
	)
	const serverProjectsQuery = useQuery(
		computed(() => ({
			...libraryServerProjectIdsQueryOptions(linkedProjectIds.value),
			enabled: linkedProjectIds.value.length > 0,
			placeholderData: keepPreviousData,
		})),
	)
	const serverProjectIds = computed(() => new Set(serverProjectsQuery.data.value ?? []))
	const getInstanceType = (instance: LibraryInstance) =>
		getLibraryInstanceType(instance, serverProjectIds.value)

	const libraryInstances = computed<LibraryInstance[]>(() =>
		[
			...installedInstances.value.filter((instance) => !instance.launch_blocked),
			...toValue(sharedInstances),
		].filter((instance) => !hiddenIds.value.has(instance.id)),
	)

	const { getSharedInstanceOwner } = useSharedInstanceOwners(libraryInstances)

	const tabInstances = computed(() =>
		libraryInstances.value.filter(
			(instance) => isUninstalledSharedInstance(instance) === (activeTab.value === 'available'),
		),
	)

	const instanceGroups = computed(() =>
		groupLibraryInstances({
			instances: tabInstances.value.filter((instance) =>
				matchesLibraryFilters(instance, filters.value, getInstanceType(instance)),
			),
			search: search.value,
			sort: sortMode.value,
			groupBy: displayState.value.group,
			tab: activeTab.value,
			groups: groups.value,
			groupOrder: groupOrder.value,
			getInstanceType,
			formatLoaderName: (loader) => formatLoader(formatMessage, loader),
			formatInstanceType: (instanceType) =>
				formatMessage(libraryInstanceTypeMessages[instanceType]),
		}),
	)

	const visibleInstanceGroups = computed(() =>
		instanceGroups.value.filter((instanceGroup) =>
			instanceGroup.id === FAVORITES_GROUP_ID
				? instanceGroup.instances.length > 0
				: instanceGroup.instances.length > 0 ||
					(!isSearching.value &&
						!hasActiveFilters.value &&
						instanceGroup.id !== UNGROUPED_GROUP_ID),
		),
	)

	return {
		instances: installedInstances,
		libraryInstances,
		tabInstances,
		activeTab,
		search,
		isSearching,
		filters,
		displayState,
		sortMode,
		groups,
		groupsLoaded,
		orderedGroupIds,
		instanceGroups,
		visibleInstanceGroups,
		isSectionCollapsed,
		setSectionCollapsed,
		getSharedInstanceOwner,
	}
}
