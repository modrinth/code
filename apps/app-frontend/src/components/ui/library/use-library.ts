import type { Labrinth } from '@modrinth/api-client'
import { formatLoader, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useEventListener, useStorage } from '@vueuse/core'
import dayjs from 'dayjs'
import {
	computed,
	inject,
	type InjectionKey,
	nextTick,
	provide,
	type Ref,
	ref,
	watch,
	watchEffect,
} from 'vue'

import { trackEvent } from '@/helpers/analytics'
import { get_project_v3_many } from '@/helpers/cache.js'
import { toError } from '@/helpers/errors'
import { install_duplicate_instance } from '@/helpers/install'
import { edit, edit_icon, remove } from '@/helpers/instance'
import {
	create_group as createInstanceGroup,
	delete_group as deleteInstanceGroup,
	FAVORITES_GROUP_ID,
	type InstanceGroupDefinition,
	list_groups as listInstanceGroups,
	MAX_INSTANCE_GROUP_NAME_LENGTH,
	rename_group as renameInstanceGroup,
	set_group_memberships as setInstanceGroupMemberships,
	set_group_order as setInstanceGroupOrder,
} from '@/helpers/instance-groups'
import type { GameInstance, InstanceIconConfig } from '@/helpers/types'

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
	{ value: 'Group', label: 'Custom group' },
	{ value: 'Instance type', label: 'Instance type' },
	{ value: 'Loader', label: 'Loader' },
	{ value: 'Game version', label: 'Game version' },
	{ value: 'None', label: 'No grouping' },
] as const

const libraryInstanceTypeLabels = {
	custom: 'Custom',
	modpack: 'Modpack',
	server: 'Server',
} as const

const libraryLoaderPriority: Record<string, number> = {
	fabric: 0,
	forge: 1,
	neoforge: 2,
}

export type LibrarySort = (typeof librarySortOptions)[number]
export type LibraryGroupBy = (typeof libraryGroupOptions)[number]['value']
export type LibraryFilters = Record<'instanceType' | 'gameVersion' | 'loader', string[]>

export type InstanceGroup = {
	id: string
	key: string
	instances: GameInstance[]
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

export type InstanceCard = {
	instance: GameInstance
	playing: boolean
	play: (event: MouseEvent | null, context: string) => Promise<void>
	stop: (event: MouseEvent | null, context: string) => Promise<void>
	addContent: () => Promise<void>
	seeInstance: () => Promise<void>
	openFolder: () => Promise<void>
}

type InstanceContextMenu = {
	showMenu: (event: MouseEvent, item: InstanceCard, options: unknown[]) => void
}

type ConfirmDeleteModal = {
	show: () => void
}

type IconEditorModal = {
	show: () => void
}

type ContextMenuSelection = {
	option: string
	item: InstanceCard
}

function createLibraryState(instances: Ref<GameInstance[]>) {
	const { handleError } = injectNotificationManager()
	const { formatMessage } = useVIntl()

	const search = ref('')
	const filters = useStorage<LibraryFilters>(
		'Instances-grid-filters',
		{
			instanceType: [],
			gameVersion: [],
			loader: [],
		},
		localStorage,
		{ mergeDefaults: true },
	)
	const serverProjectIds = ref(new Set<string>())
	const libraryGroups = ref<InstanceGroupDefinition[]>([])
	const libraryGroupsLoaded = ref(false)
	const isNewGroupModalOpen = ref(false)
	const newGroupName = ref('')
	const newGroupSearch = ref('')
	const selectedNewGroupInstanceIds = ref(new Set<string>())
	const isGroupInstancesModalOpen = ref(false)
	const groupInstancesModalGroupId = ref<string | null>(null)
	const groupInstancesSearch = ref('')
	const selectedGroupInstanceIds = ref(new Set<string>())
	const savingGroupInstances = ref(false)
	const selectedLibraryInstances = ref(new Map<string, LibraryInstanceSelection>())
	const isLibraryInstanceSelectionActive = computed(() => selectedLibraryInstances.value.size > 0)
	const creatingGroup = ref(false)
	const reorderingGroups = ref(false)
	const groupIdPendingNameEdit = ref<string | null>(null)
	const activeInstanceGroupDrag = ref<ActiveInstanceGroupDrag | null>(null)
	const pendingInstanceGroupIds = ref(new Map<string, string[]>())
	const activeDraggedInstanceKeys = computed(
		() =>
			new Set(activeInstanceGroupDrag.value?.instances.map(getLibraryInstanceSelectionKey) ?? []),
	)
	const instanceGroupDragTarget = ref<string | null>(null)
	const instanceGroupDragPointer = ref({ x: 0, y: 0 })
	const isAddingInstanceToGroup = ref(false)
	const instanceOptions = ref<InstanceContextMenu | null>(null)
	const currentDeleteInstanceId = ref<string | null>(null)
	const currentDeleteInstances = computed(() =>
		instances.value.filter((instance) => instance.id === currentDeleteInstanceId.value),
	)
	const currentContextGroupId = ref<string | null>(null)
	const confirmDeleteModal = ref<ConfirmDeleteModal | null>(null)
	const iconEditorModal = ref<IconEditorModal | null>(null)
	const currentIconEditorInstanceId = ref<string | null>(null)
	const currentIconEditorInstance = computed(
		() =>
			instances.value.find((instance) => instance.id === currentIconEditorInstanceId.value) ?? null,
	)

	const displayState = useStorage<{
		group: LibraryGroupBy
		sortBy: LibrarySort
		collapsedGroups: string[]
		ungroupedGroupPosition: number
	}>(
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

	if (!libraryGroupOptions.some((option) => option.value === displayState.value.group)) {
		displayState.value.group = 'Group'
	}

	const linkedInstances = computed(() => instances.value.filter((instance) => instance.link))
	const isSearching = computed(() => search.value.length > 0)
	const collapsedSectionKeys = computed(() => new Set(displayState.value.collapsedGroups))
	const groupNames = computed(
		() =>
			new Set(
				libraryGroups.value
					.filter((group) => group.id !== FAVORITES_GROUP_ID)
					.map((group) => group.name)
					.map((group) => group.trim())
					.filter((group) => group && group.toLowerCase() !== 'none'),
			),
	)
	const existingGroupNames = computed(
		() => new Set(['none', ...Array.from(groupNames.value, (group) => group.toLowerCase())]),
	)
	const normalizedNewGroupName = computed(() =>
		newGroupName.value.trim().substring(0, MAX_INSTANCE_GROUP_NAME_LENGTH),
	)
	const newGroupInstances = computed(() => {
		const query = newGroupSearch.value.trim().toLowerCase()

		return instances.value
			.filter((instance) => !query || instance.name.toLowerCase().includes(query))
			.slice()
			.sort((a, b) => {
				const groupedDifference = Number(a.group_ids.length > 0) - Number(b.group_ids.length > 0)
				if (groupedDifference !== 0) return groupedDifference

				return a.name.localeCompare(b.name)
			})
	})
	const groupInstancesModalGroup = computed(() => {
		if (groupInstancesModalGroupId.value === 'group:none') {
			return { id: 'group:none', name: 'None' }
		}

		return (
			libraryGroups.value.find((group) => group.id === groupInstancesModalGroupId.value) ?? null
		)
	})
	const groupInstances = computed(() => {
		const query = groupInstancesSearch.value.trim().toLowerCase()

		return instances.value
			.filter((instance) => !query || instance.name.toLowerCase().includes(query))
			.slice()
			.sort((a, b) => a.name.localeCompare(b.name))
	})
	const canCreateGroup = computed(
		() => normalizedNewGroupName.value.length > 0 && !creatingGroup.value,
	)
	const customLibraryGroups = computed(() =>
		libraryGroups.value.filter((group) => group.id !== FAVORITES_GROUP_ID),
	)
	const orderedLibraryGroupIds = computed(() => {
		const groupIds = customLibraryGroups.value.map((group) => group.id)
		const storedUngroupedGroupPosition = displayState.value.ungroupedGroupPosition
		const ungroupedGroupPosition = Math.min(
			Math.max(
				Number.isFinite(storedUngroupedGroupPosition)
					? Math.trunc(storedUngroupedGroupPosition)
					: Number.MAX_SAFE_INTEGER,
				0,
			),
			groupIds.length,
		)
		groupIds.splice(ungroupedGroupPosition, 0, 'group:none')
		return groupIds
	})
	const libraryGroupOrder = computed(
		() => new Map(orderedLibraryGroupIds.value.map((groupId, index) => [groupId, index])),
	)

	const refreshGroups = async () => {
		try {
			libraryGroups.value = await listInstanceGroups()
		} catch (error) {
			handleError(toError(error))
		} finally {
			libraryGroupsLoaded.value = true
		}
	}

	void refreshGroups()

	watchEffect(async () => {
		const projectIds = [
			...new Set(
				linkedInstances.value.flatMap((instance) =>
					instance.link?.project_id ? [instance.link.project_id] : [],
				),
			),
		]

		if (projectIds.length === 0) {
			serverProjectIds.value = new Set()
			return
		}

		try {
			const projects = (await get_project_v3_many(
				projectIds,
				'must_revalidate',
			)) as Array<Labrinth.Projects.v3.Project | null>
			serverProjectIds.value = new Set(
				projects
					.filter(
						(project): project is Labrinth.Projects.v3.Project => project?.minecraft_server != null,
					)
					.map((project) => project.id),
			)
		} catch {
			serverProjectIds.value = new Set()
		}
	})

	const getInstanceType = (instance: GameInstance) => {
		if (!instance.link) return 'custom'
		if (serverProjectIds.value.has(instance.link.project_id ?? '')) return 'server'
		return 'modpack'
	}
	const getEffectiveInstanceGroupIds = (instance: GameInstance) =>
		pendingInstanceGroupIds.value.get(instance.id) ?? instance.group_ids
	const haveSameGroupIds = (first: string[], second: string[]) =>
		first.length === second.length && first.every((groupId) => second.includes(groupId))

	watch(
		instances,
		(currentInstances) => {
			if (pendingInstanceGroupIds.value.size === 0) return

			const nextPendingInstanceGroupIds = new Map(pendingInstanceGroupIds.value)
			for (const instance of currentInstances) {
				const pendingGroupIds = nextPendingInstanceGroupIds.get(instance.id)
				if (pendingGroupIds && haveSameGroupIds(pendingGroupIds, instance.group_ids)) {
					nextPendingInstanceGroupIds.delete(instance.id)
				}
			}

			if (nextPendingInstanceGroupIds.size !== pendingInstanceGroupIds.value.size) {
				pendingInstanceGroupIds.value = nextPendingInstanceGroupIds
			}
		},
		{ flush: 'sync' },
	)

	const filteredInstances = computed(() =>
		instances.value.filter((instance) => {
			const instanceTypeMatches =
				filters.value.instanceType.length === 0 ||
				filters.value.instanceType.includes(getInstanceType(instance))
			const gameVersionMatches =
				filters.value.gameVersion.length === 0 ||
				filters.value.gameVersion.includes(instance.game_version)
			const loaderMatches =
				filters.value.loader.length === 0 || filters.value.loader.includes(instance.loader)

			return instanceTypeMatches && gameVersionMatches && loaderMatches
		}),
	)

	const instanceGroups = computed<InstanceGroup[]>(() => {
		const visibleInstances = filteredInstances.value.filter((instance) =>
			instance.name.toLowerCase().includes(search.value.toLowerCase()),
		)

		switch (displayState.value.sortBy) {
			case 'Name':
				visibleInstances.sort((a, b) => a.name.localeCompare(b.name))
				break
			case 'Loader':
				visibleInstances.sort((a, b) =>
					formatLoader(formatMessage, a.loader).localeCompare(
						formatLoader(formatMessage, b.loader),
					),
				)
				break
			case 'Game version':
				visibleInstances.sort((a, b) =>
					a.game_version.localeCompare(b.game_version, undefined, { numeric: true }),
				)
				break
			case 'Last played':
				visibleInstances.sort((a, b) => dayjs(b.last_played ?? 0).diff(dayjs(a.last_played ?? 0)))
				break
			case 'Hours played':
				visibleInstances.sort(
					(a, b) =>
						b.recent_time_played +
						b.submitted_time_played -
						(a.recent_time_played + a.submitted_time_played),
				)
				break
			case 'Date created':
				visibleInstances.sort((a, b) => dayjs(b.created).diff(dayjs(a.created)))
				break
			case 'Date modified':
				visibleInstances.sort((a, b) => dayjs(b.modified).diff(dayjs(a.modified)))
				break
		}

		const groupedInstances = new Map<string, { name: string; instances: GameInstance[] }>()
		const addToGroup = (id: string, name: string, instance: GameInstance) => {
			const group = groupedInstances.get(id) ?? { name, instances: [] }
			group.instances.push(instance)
			groupedInstances.set(id, group)
		}
		const groupsById = new Map(libraryGroups.value.map((group) => [group.id, group]))

		for (const instance of visibleInstances) {
			const instanceGroupIds = getEffectiveInstanceGroupIds(instance)
			switch (displayState.value.group) {
				case 'Instance type':
					{
						const instanceType = getInstanceType(instance)
						addToGroup(
							`Instance type:${instanceType}`,
							libraryInstanceTypeLabels[instanceType],
							instance,
						)
					}
					break
				case 'Loader':
					{
						const name = formatLoader(formatMessage, instance.loader)
						addToGroup(`Loader:${name}`, name, instance)
					}
					break
				case 'Game version':
					addToGroup(`Game version:${instance.game_version}`, instance.game_version, instance)
					break
				case 'Group':
					if (instanceGroupIds.length === 0) {
						addToGroup('group:none', 'None', instance)
					} else {
						for (const groupId of instanceGroupIds) {
							const group = groupsById.get(groupId)
							addToGroup(groupId, group?.name ?? groupId, instance)
						}
					}
					break
				case 'None':
					addToGroup('None:None', 'None', instance)
					break
			}
		}

		if (displayState.value.group === 'Group') {
			if (!groupedInstances.has('group:none')) {
				groupedInstances.set('group:none', { name: 'None', instances: [] })
			}

			for (const group of libraryGroups.value) {
				if (!groupedInstances.has(group.id)) {
					groupedInstances.set(group.id, { name: group.name, instances: [] })
				}
			}
		}

		const groups = Array.from(groupedInstances, ([id, group]) => ({
			id,
			key: group.name,
			instances: group.instances,
		}))

		if (displayState.value.sortBy === 'Name') {
			groups.sort((a, b) => a.key.localeCompare(b.key) || a.id.localeCompare(b.id))
		}

		if (displayState.value.group === 'Loader') {
			groups.sort((a, b) => {
				const aPriority = libraryLoaderPriority[a.key.toLowerCase()] ?? Number.MAX_SAFE_INTEGER
				const bPriority = libraryLoaderPriority[b.key.toLowerCase()] ?? Number.MAX_SAFE_INTEGER
				return aPriority - bPriority || a.key.localeCompare(b.key) || a.id.localeCompare(b.id)
			})
		}

		if (displayState.value.group === 'Instance type') {
			groups.sort((a, b) => a.key.localeCompare(b.key) || a.id.localeCompare(b.id))
		}

		if (displayState.value.group === 'Game version') {
			groups.sort((a, b) => b.key.localeCompare(a.key, undefined, { numeric: true }))
		}

		if (displayState.value.group === 'Group') {
			groups.sort((a, b) => {
				if (a.id === b.id) return 0
				if (a.id === FAVORITES_GROUP_ID) return -1
				if (b.id === FAVORITES_GROUP_ID) return 1

				const aOrder = libraryGroupOrder.value.get(a.id) ?? Number.MAX_SAFE_INTEGER
				const bOrder = libraryGroupOrder.value.get(b.id) ?? Number.MAX_SAFE_INTEGER
				return aOrder - bOrder || a.key.localeCompare(b.key) || a.id.localeCompare(b.id)
			})
		}

		return groups
	})

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

	const normalizeInstanceGroupId = (groupId: string) => (groupId === 'group:none' ? null : groupId)

	const updateInstanceGroupDrag = (
		pointer?: { x: number; y: number },
		altKey = isAddingInstanceToGroup.value,
	) => {
		if (pointer) {
			instanceGroupDragPointer.value = {
				x: pointer.x,
				y: pointer.y,
			}
		}
		isAddingInstanceToGroup.value = altKey
	}

	const startInstanceGroupDrag = (
		instanceId: string,
		groupId: string,
		pointer?: { x: number; y: number },
		altKey = false,
	) => {
		if (displayState.value.group !== 'Group') return

		const primaryInstance = { instanceId, groupId }
		const primaryInstanceKey = getLibraryInstanceSelectionKey(primaryInstance)
		const draggedInstances = selectedLibraryInstances.value.has(primaryInstanceKey)
			? [
					primaryInstance,
					...[...selectedLibraryInstances.value.values()].filter(
						(selectedInstance) =>
							getLibraryInstanceSelectionKey(selectedInstance) !== primaryInstanceKey,
					),
				].filter((selectedInstance) =>
					instances.value.some((instance) => instance.id === selectedInstance.instanceId),
				)
			: [primaryInstance]

		activeInstanceGroupDrag.value = {
			instances: draggedInstances,
			primaryInstanceId: instanceId,
			fromGroup: normalizeInstanceGroupId(groupId),
		}
		updateInstanceGroupDrag(pointer, altKey)
	}

	const finishInstanceGroupDrag = () => {
		activeInstanceGroupDrag.value = null
		instanceGroupDragTarget.value = null
		isAddingInstanceToGroup.value = false
	}

	const setInstanceGroupDragTarget = (groupId: string | null) => {
		instanceGroupDragTarget.value = groupId
	}

	const getInstanceGroupDropState = (groupId: string) => {
		const drag = activeInstanceGroupDrag.value
		const draggedSelectionsByInstanceId = new Map<string, LibraryInstanceSelection[]>()
		for (const selection of drag?.instances ?? []) {
			const selections = draggedSelectionsByInstanceId.get(selection.instanceId) ?? []
			selections.push(selection)
			draggedSelectionsByInstanceId.set(selection.instanceId, selections)
		}
		const draggedInstanceIds = new Set(
			drag?.instances.map((selection) => selection.instanceId) ?? [],
		)
		const draggedInstances = instances.value.filter((instance) =>
			draggedInstanceIds.has(instance.id),
		)
		const toGroup = normalizeInstanceGroupId(groupId)
		const operation = isAddingInstanceToGroup.value ? 'add' : 'move'
		const canAddToGroup = operation !== 'add' || toGroup !== null
		const hasChanges = draggedInstances.some((instance) => {
			const instanceGroupIds = getEffectiveInstanceGroupIds(instance)
			if (operation === 'add') {
				return toGroup !== null && !instanceGroupIds.includes(toGroup)
			}

			if (toGroup !== null) {
				return !instanceGroupIds.includes(toGroup)
			}

			const sourceGroupIds = new Set(
				(draggedSelectionsByInstanceId.get(instance.id) ?? [])
					.map((selection) => normalizeInstanceGroupId(selection.groupId))
					.filter((sourceGroupId): sourceGroupId is string => sourceGroupId !== null),
			)
			return instanceGroupIds.some((sourceGroupId) => sourceGroupIds.has(sourceGroupId))
		})

		return {
			alreadyInGroup: draggedInstances.length > 0 && !hasChanges,
			canDrop: !!drag && draggedInstances.length > 0 && hasChanges && canAddToGroup,
			operation,
		} as const
	}

	const instanceGroupDragStatus = computed(() => {
		const target = instanceGroupDragTarget.value
		if (target) {
			const dropState = getInstanceGroupDropState(target)
			const isSourceGroup =
				normalizeInstanceGroupId(target) === activeInstanceGroupDrag.value?.fromGroup
			if (!isSourceGroup && !dropState.canDrop && dropState.alreadyInGroup) {
				return 'Already in this group'
			}
			if (!dropState.canDrop || dropState.operation !== 'add') return

			const count = new Set(
				activeInstanceGroupDrag.value?.instances.map((selection) => selection.instanceId) ?? [],
			).size
			return count > 1 ? `Duplicate ${count} instances to group` : 'Duplicate into group'
		}

		if (isAddingInstanceToGroup.value) return 'Duplicate to group'
		return
	})

	const moveDraggedInstancesToGroup = async (groupId: string, addToGroup = false) => {
		const drag = activeInstanceGroupDrag.value
		const toGroup = normalizeInstanceGroupId(groupId)
		if (!drag) return false

		const dropState = getInstanceGroupDropState(groupId)
		if (!dropState.canDrop) return false

		const shouldAdd = addToGroup && toGroup !== null
		const draggedSelectionsByInstanceId = new Map<string, LibraryInstanceSelection[]>()
		for (const selection of drag.instances) {
			const selections = draggedSelectionsByInstanceId.get(selection.instanceId) ?? []
			selections.push(selection)
			draggedSelectionsByInstanceId.set(selection.instanceId, selections)
		}
		const draggedInstanceIds = new Set(draggedSelectionsByInstanceId.keys())
		const draggedInstances = instances.value.filter((instance) =>
			draggedInstanceIds.has(instance.id),
		)
		const operations = draggedInstances.flatMap((instance) => {
			const instanceGroupIds = getEffectiveInstanceGroupIds(instance)
			if (toGroup && instanceGroupIds.includes(toGroup)) {
				return []
			}

			const selections = draggedSelectionsByInstanceId.get(instance.id) ?? []
			let nextGroupIds = [...instanceGroupIds]
			if (shouldAdd && toGroup) {
				nextGroupIds.push(toGroup)
			} else {
				const sourceGroupIds = new Set(
					selections
						.map((selection) => normalizeInstanceGroupId(selection.groupId))
						.filter((sourceGroupId): sourceGroupId is string => sourceGroupId !== null),
				)
				nextGroupIds = nextGroupIds.filter((sourceGroupId) => !sourceGroupIds.has(sourceGroupId))
				if (toGroup) {
					nextGroupIds.push(toGroup)
				}
			}

			return [
				{
					instanceId: instance.id,
					nextGroupIds,
				},
			]
		})
		const nextPendingInstanceGroupIds = new Map(pendingInstanceGroupIds.value)
		for (const operation of operations) {
			nextPendingInstanceGroupIds.set(operation.instanceId, operation.nextGroupIds)
		}
		pendingInstanceGroupIds.value = nextPendingInstanceGroupIds

		try {
			await setInstanceGroupMemberships(
				operations.map((operation) => ({
					instance_id: operation.instanceId,
					group_ids: operation.nextGroupIds,
				})),
			)
		} catch (error) {
			const pendingInstanceGroupIdsAfterFailure = new Map(pendingInstanceGroupIds.value)
			for (const operation of operations) {
				const pendingGroupIds = pendingInstanceGroupIdsAfterFailure.get(operation.instanceId)
				if (pendingGroupIds && haveSameGroupIds(pendingGroupIds, operation.nextGroupIds)) {
					pendingInstanceGroupIdsAfterFailure.delete(operation.instanceId)
				}
			}
			pendingInstanceGroupIds.value = pendingInstanceGroupIdsAfterFailure
			handleError(toError(error))
			return false
		}

		selectedLibraryInstances.value = new Map()

		return operations.length > 0
	}

	useEventListener(window, 'keydown', (event) => {
		if (event.key === 'Alt' && activeInstanceGroupDrag.value) {
			isAddingInstanceToGroup.value = true
		}
	})
	useEventListener(window, 'keyup', (event) => {
		if (event.key === 'Alt') {
			isAddingInstanceToGroup.value = false
		}
	})
	useEventListener(window, 'blur', () => {
		isAddingInstanceToGroup.value = false
	})

	const getDefaultNewGroupName = () => {
		let groupNumber = groupNames.value.size + 1
		while (existingGroupNames.value.has(`group ${groupNumber}`.toLowerCase())) {
			groupNumber++
		}

		return `Group ${groupNumber}`
	}

	const openNewGroupModal = (instanceIds: Iterable<string> = []) => {
		newGroupName.value = getDefaultNewGroupName()
		newGroupSearch.value = ''
		selectedNewGroupInstanceIds.value = new Set(instanceIds)
		isNewGroupModalOpen.value = true
	}

	const closeNewGroupModal = () => {
		isNewGroupModalOpen.value = false
	}

	const openGroupInstancesModal = (groupId: string) => {
		const group = libraryGroups.value.find((candidate) => candidate.id === groupId)
		if (!group && groupId !== 'group:none') return

		groupInstancesModalGroupId.value = groupId
		groupInstancesSearch.value = ''
		selectedGroupInstanceIds.value = new Set(
			instances.value
				.filter((instance) =>
					groupId === 'group:none'
						? instance.group_ids.length === 0
						: instance.group_ids.includes(groupId),
				)
				.map((instance) => instance.id),
		)
		isGroupInstancesModalOpen.value = true
	}

	const closeGroupInstancesModal = () => {
		isGroupInstancesModalOpen.value = false
		groupInstancesModalGroupId.value = null
	}

	const toggleGroupInstance = (instanceId: string) => {
		const selectedIds = new Set(selectedGroupInstanceIds.value)

		if (selectedIds.has(instanceId)) {
			if (groupInstancesModalGroupId.value === 'group:none') return
			selectedIds.delete(instanceId)
		} else {
			selectedIds.add(instanceId)
		}

		selectedGroupInstanceIds.value = selectedIds
	}

	const saveGroupInstances = async () => {
		const groupId = groupInstancesModalGroupId.value
		if (!groupId || savingGroupInstances.value) return false

		const isUngrouped = groupId === 'group:none'
		const changedInstances = instances.value.filter((instance) => {
			const isSelected = selectedGroupInstanceIds.value.has(instance.id)
			return isUngrouped
				? isSelected && instance.group_ids.length > 0
				: instance.group_ids.includes(groupId) !== isSelected
		})
		const operations = changedInstances.map((instance) => {
			const shouldIncludeGroup = selectedGroupInstanceIds.value.has(instance.id)
			const nextGroupIds = isUngrouped
				? []
				: shouldIncludeGroup
					? [...instance.group_ids, groupId]
					: instance.group_ids.filter((instanceGroupId) => instanceGroupId !== groupId)

			return {
				instance,
				shouldIncludeGroup,
				nextGroupIds,
			}
		})
		savingGroupInstances.value = true

		try {
			await setInstanceGroupMemberships(
				operations.map(({ instance, nextGroupIds }) => ({
					instance_id: instance.id,
					group_ids: nextGroupIds,
				})),
			)

			const nextSelectedInstances = new Map(selectedLibraryInstances.value)
			for (const { instance, shouldIncludeGroup, nextGroupIds } of operations) {
				const instanceSelections = [...nextSelectedInstances.values()].filter(
					(selection) => selection.instanceId === instance.id,
				)
				if (shouldIncludeGroup && instanceSelections.length > 0) {
					for (const selection of instanceSelections) {
						nextSelectedInstances.delete(getLibraryInstanceSelectionKey(selection))
					}
					const destinationSelection = {
						instanceId: instance.id,
						groupId,
					}
					nextSelectedInstances.set(
						getLibraryInstanceSelectionKey(destinationSelection),
						destinationSelection,
					)
					continue
				}

				const validSelectionGroupIds = new Set(
					nextGroupIds.length > 0 ? nextGroupIds : ['group:none'],
				)
				const invalidSelections = instanceSelections.filter(
					(selection) => !validSelectionGroupIds.has(selection.groupId),
				)
				for (const selection of invalidSelections) {
					nextSelectedInstances.delete(getLibraryInstanceSelectionKey(selection))
				}

				const hasValidSelection = [...nextSelectedInstances.values()].some(
					(selection) => selection.instanceId === instance.id,
				)
				if (invalidSelections.length > 0 && !hasValidSelection) {
					const replacementSelection = {
						instanceId: instance.id,
						groupId: nextGroupIds[0] ?? 'group:none',
					}
					nextSelectedInstances.set(
						getLibraryInstanceSelectionKey(replacementSelection),
						replacementSelection,
					)
				}
			}
			selectedLibraryInstances.value = nextSelectedInstances
			return true
		} catch (error) {
			handleError(toError(error))
			return false
		} finally {
			savingGroupInstances.value = false
		}
	}

	const toggleNewGroupInstance = (instanceId: string) => {
		const selectedIds = new Set(selectedNewGroupInstanceIds.value)

		if (selectedIds.has(instanceId)) {
			selectedIds.delete(instanceId)
		} else {
			selectedIds.add(instanceId)
		}

		selectedNewGroupInstanceIds.value = selectedIds
	}

	const clearLibraryInstanceSelection = () => {
		selectedLibraryInstances.value = new Map()
	}

	watch(() => displayState.value.group, clearLibraryInstanceSelection)

	const setSelectedLibraryInstances = (selections: Iterable<LibraryInstanceSelection>) => {
		selectedLibraryInstances.value = new Map(
			[...selections].map((selection) => [getLibraryInstanceSelectionKey(selection), selection]),
		)
	}

	const toggleLibraryInstanceSelection = (selection: LibraryInstanceSelection) => {
		const selectedInstances = new Map(selectedLibraryInstances.value)
		const selectionKey = getLibraryInstanceSelectionKey(selection)

		if (selectedInstances.has(selectionKey)) {
			selectedInstances.delete(selectionKey)
		} else {
			selectedInstances.set(selectionKey, selection)
		}

		selectedLibraryInstances.value = selectedInstances
	}

	useEventListener(window, 'keydown', (event) => {
		if (
			event.key === 'Escape' &&
			!event.defaultPrevented &&
			isLibraryInstanceSelectionActive.value
		) {
			clearLibraryInstanceSelection()
		}
	})

	const createGroup = async () => {
		if (!canCreateGroup.value) return false

		creatingGroup.value = true

		try {
			const group = await createInstanceGroup(normalizedNewGroupName.value)
			libraryGroups.value = [
				group,
				...libraryGroups.value.filter((existingGroup) => existingGroup.id !== group.id),
			]

			await setInstanceGroupMemberships(
				instances.value
					.filter((instance) => selectedNewGroupInstanceIds.value.has(instance.id))
					.map((instance) => ({
						instance_id: instance.id,
						group_ids: [...instance.group_ids, group.id],
					})),
			)
			return true
		} catch (error) {
			handleError(toError(error))
			await refreshGroups()
			return false
		} finally {
			creatingGroup.value = false
		}
	}

	const createDefaultGroup = async (instanceIds: Iterable<string> = []) => {
		if (creatingGroup.value) return false

		creatingGroup.value = true

		try {
			const group = await createInstanceGroup(getDefaultNewGroupName())
			const selectedInstanceIds = new Set(instanceIds)
			const instancesToAdd = instances.value.filter((instance) =>
				selectedInstanceIds.has(instance.id),
			)
			await setInstanceGroupMemberships(
				instancesToAdd.map((instance) => ({
					instance_id: instance.id,
					group_ids: [...instance.group_ids, group.id],
				})),
			)

			libraryGroups.value = [
				group,
				...libraryGroups.value.filter((existingGroup) => existingGroup.id !== group.id),
			]

			displayState.value.group = 'Group'
			groupIdPendingNameEdit.value = group.id
			return true
		} catch (error) {
			handleError(toError(error))
			await refreshGroups()
			return false
		} finally {
			creatingGroup.value = false
		}
	}

	const completePendingGroupNameEdit = (groupId: string) => {
		if (groupIdPendingNameEdit.value === groupId) {
			groupIdPendingNameEdit.value = null
		}
	}

	const deleteGroup = async (groupId: string) => {
		try {
			await deleteInstanceGroup(groupId)
			libraryGroups.value = libraryGroups.value.filter((group) => group.id !== groupId)
			return true
		} catch (error) {
			handleError(toError(error))
			await refreshGroups()
			return false
		}
	}

	const renameGroup = async (groupId: string, newName: string) => {
		const normalizedNewName = newName.trim()
		const currentGroup = libraryGroups.value.find((group) => group.id === groupId)
		if (currentGroup?.name === normalizedNewName) return true

		try {
			const renamedGroup = await renameInstanceGroup(groupId, normalizedNewName)
			libraryGroups.value = libraryGroups.value.map((group) =>
				group.id === groupId ? renamedGroup : group,
			)
			return true
		} catch (error) {
			handleError(toError(error))
			await refreshGroups()
			return false
		}
	}

	const canMoveGroupUp = (groupId: string) =>
		!reorderingGroups.value &&
		orderedLibraryGroupIds.value.findIndex((orderedGroupId) => orderedGroupId === groupId) > 0

	const canMoveGroupDown = (groupId: string) => {
		const groupIndex = orderedLibraryGroupIds.value.findIndex(
			(orderedGroupId) => orderedGroupId === groupId,
		)
		return (
			!reorderingGroups.value &&
			groupIndex >= 0 &&
			groupIndex < orderedLibraryGroupIds.value.length - 1
		)
	}

	const reorderGroups = async (orderedGroupIds: string[]) => {
		if (reorderingGroups.value) return false

		const previousGroups = libraryGroups.value
		const previousUngroupedGroupPosition = displayState.value.ungroupedGroupPosition
		const customGroupsById = new Map(customLibraryGroups.value.map((group) => [group.id, group]))
		const reorderableGroupIds = new Set([...customGroupsById.keys(), 'group:none'])
		const orderedGroupIdSet = new Set(orderedGroupIds)

		if (
			orderedGroupIdSet.size !== orderedGroupIds.length ||
			orderedGroupIds.some((groupId) => !reorderableGroupIds.has(groupId))
		) {
			return false
		}

		let orderedGroupIndex = 0
		const reorderedGroupIds = orderedLibraryGroupIds.value.map((groupId) =>
			orderedGroupIdSet.has(groupId) ? orderedGroupIds[orderedGroupIndex++] : groupId,
		)

		if (
			reorderedGroupIds.every((groupId, index) => groupId === orderedLibraryGroupIds.value[index])
		) {
			return false
		}

		const reorderedCustomGroups = reorderedGroupIds
			.filter((groupId) => groupId !== 'group:none')
			.map((groupId) => customGroupsById.get(groupId)!)
		const customGroupOrderChanged = reorderedCustomGroups.some(
			(group, index) => group !== customLibraryGroups.value[index],
		)
		const favoriteGroups = previousGroups.filter((group) => group.id === FAVORITES_GROUP_ID)
		libraryGroups.value = [...favoriteGroups, ...reorderedCustomGroups]
		displayState.value.ungroupedGroupPosition = reorderedGroupIds.indexOf('group:none')
		reorderingGroups.value = true

		try {
			if (customGroupOrderChanged) {
				await setInstanceGroupOrder(reorderedCustomGroups.map((group) => group.id))
			}
			return true
		} catch (error) {
			libraryGroups.value = previousGroups
			displayState.value.ungroupedGroupPosition = previousUngroupedGroupPosition
			handleError(toError(error))
			await refreshGroups()
			return false
		} finally {
			reorderingGroups.value = false
		}
	}

	const moveGroup = async (groupId: string, direction: -1 | 1) => {
		const orderedGroupIds = [...orderedLibraryGroupIds.value]
		const groupIndex = orderedGroupIds.indexOf(groupId)
		const targetIndex = groupIndex + direction

		if (groupIndex < 0 || targetIndex < 0 || targetIndex >= orderedGroupIds.length) {
			return false
		}

		const targetGroupId = orderedGroupIds[targetIndex]
		orderedGroupIds[targetIndex] = orderedGroupIds[groupIndex]
		orderedGroupIds[groupIndex] = targetGroupId

		return await reorderGroups(orderedGroupIds)
	}

	const deleteInstance = async () => {
		if (!currentDeleteInstanceId.value) return

		await remove(currentDeleteInstanceId.value).catch((error) => handleError(toError(error)))
		currentDeleteInstanceId.value = null
	}

	const duplicateInstance = async (instanceId: string) => {
		await install_duplicate_instance(instanceId).catch((error) => handleError(toError(error)))
	}

	const selectInstanceIcon = async (item: InstanceCard) => {
		const iconPath = await openDialog({
			multiple: false,
			filters: [
				{
					name: 'Image',
					extensions: ['png', 'jpeg', 'svg', 'webp', 'gif', 'jpg'],
				},
			],
		})

		if (!iconPath) return

		try {
			await edit_icon(item.instance.id, iconPath)
			trackEvent('InstanceSetIcon')
		} catch (error) {
			handleError(toError(error))
		}
	}

	const removeInstanceIcon = async (item: InstanceCard) => {
		try {
			await edit_icon(item.instance.id, null)
			trackEvent('InstanceRemoveIcon')
		} catch (error) {
			handleError(toError(error))
		}
	}

	const openInstanceIconEditor = async (item: InstanceCard) => {
		currentIconEditorInstanceId.value = item.instance.id
		await nextTick()
		iconEditorModal.value?.show()
		trackEvent(item.instance.icon_config ? 'InstanceEditCreatedIcon' : 'InstanceCreateIcon')
	}

	const handleInstanceIconSaved = (_iconPath: string, _config: InstanceIconConfig) => {
		trackEvent('InstanceSaveCreatedIcon')
	}

	const handleInstanceContextMenu = (
		event: MouseEvent,
		item: InstanceCard,
		instanceGroupId: string,
	) => {
		currentContextGroupId.value =
			displayState.value.group === 'Group' &&
			instanceGroupId !== 'group:none' &&
			instanceGroupId !== FAVORITES_GROUP_ID
				? instanceGroupId
				: null

		const baseOptions = [
			{
				name: item.instance.group_ids.includes(FAVORITES_GROUP_ID)
					? 'remove_from_favorites'
					: 'add_to_favorites',
			},
			{ type: 'divider' },
			...(!item.instance.quarantined && !item.instance.link
				? [{ name: 'add_content' }, { type: 'divider' }]
				: []),
			{ name: 'edit' },
			{ name: 'duplicate' },
			{ name: 'open' },
			{ name: 'copy' },
			{
				name: 'edit_icon',
				children: [
					{ name: item.instance.icon_path ? 'replace_icon' : 'select_icon' },
					{ name: item.instance.icon_config ? 'edit_created_icon' : 'create_icon' },
					...(item.instance.icon_path ? [{ name: 'remove_icon' }] : []),
				],
			},
			...(currentContextGroupId.value
				? [{ name: 'remove_from_group' }, { type: 'divider' }]
				: [{ type: 'divider' }]),
			{ name: 'delete', color: 'danger' },
		]

		instanceOptions.value?.showMenu(
			event,
			item,
			item.playing
				? [{ name: 'stop', color: 'danger' }, ...baseOptions]
				: [
						...(item.instance.quarantined ? [] : [{ name: 'play', color: 'primary' }]),
						...baseOptions,
					],
		)
	}

	const handleInstanceOption = async ({ option, item }: ContextMenuSelection) => {
		switch (option) {
			case 'play':
				await item.play(null, 'InstanceGridContextMenu')
				break
			case 'stop':
				await item.stop(null, 'InstanceGridContextMenu')
				break
			case 'add_content':
				await item.addContent()
				break
			case 'add_to_favorites':
				await edit(item.instance.id, {
					group_ids: [...new Set([...item.instance.group_ids, FAVORITES_GROUP_ID])],
				}).catch((error) => handleError(toError(error)))
				break
			case 'remove_from_favorites':
				await edit(item.instance.id, {
					group_ids: item.instance.group_ids.filter(
						(instanceGroupId) => instanceGroupId !== FAVORITES_GROUP_ID,
					),
				}).catch((error) => handleError(toError(error)))
				break
			case 'edit':
				await item.seeInstance()
				break
			case 'select_icon':
			case 'replace_icon':
				await selectInstanceIcon(item)
				break
			case 'create_icon':
			case 'edit_created_icon':
				await openInstanceIconEditor(item)
				break
			case 'remove_icon':
				await removeInstanceIcon(item)
				break
			case 'duplicate':
				if (item.instance.install_stage === 'installed') {
					await duplicateInstance(item.instance.id)
				}
				break
			case 'open':
				await item.openFolder()
				break
			case 'copy':
				await navigator.clipboard.writeText(item.instance.id)
				break
			case 'remove_from_group':
				if (currentContextGroupId.value) {
					const groupId = currentContextGroupId.value
					await edit(item.instance.id, {
						group_ids: item.instance.group_ids.filter(
							(instanceGroupId) => instanceGroupId !== groupId,
						),
					}).catch((error) => handleError(toError(error)))
				}
				break
			case 'delete':
				currentDeleteInstanceId.value = item.instance.id
				confirmDeleteModal.value?.show()
				break
		}
	}

	return {
		instances,
		libraryGroups,
		libraryGroupsLoaded,
		search,
		isSearching,
		filters,
		displayState,
		instanceGroups,
		isNewGroupModalOpen,
		newGroupName,
		newGroupSearch,
		selectedNewGroupInstanceIds,
		isGroupInstancesModalOpen,
		groupInstancesModalGroup,
		groupInstancesSearch,
		selectedGroupInstanceIds,
		savingGroupInstances,
		selectedLibraryInstances,
		isLibraryInstanceSelectionActive,
		activeInstanceGroupDrag,
		activeDraggedInstanceKeys,
		instanceGroupDragTarget,
		instanceGroupDragPointer,
		instanceGroupDragStatus,
		isAddingInstanceToGroup,
		creatingGroup,
		reorderingGroups,
		groupIdPendingNameEdit,
		newGroupInstances,
		groupInstances,
		canCreateGroup,
		instanceOptions,
		confirmDeleteModal,
		iconEditorModal,
		currentIconEditorInstance,
		currentDeleteInstances,
		isSectionCollapsed,
		setSectionCollapsed,
		startInstanceGroupDrag,
		updateInstanceGroupDrag,
		finishInstanceGroupDrag,
		setInstanceGroupDragTarget,
		getInstanceGroupDropState,
		moveDraggedInstancesToGroup,
		openNewGroupModal,
		closeNewGroupModal,
		openGroupInstancesModal,
		closeGroupInstancesModal,
		toggleNewGroupInstance,
		toggleGroupInstance,
		saveGroupInstances,
		clearLibraryInstanceSelection,
		setSelectedLibraryInstances,
		toggleLibraryInstanceSelection,
		createGroup,
		createDefaultGroup,
		completePendingGroupNameEdit,
		deleteGroup,
		renameGroup,
		canMoveGroupUp,
		canMoveGroupDown,
		reorderGroups,
		moveGroup,
		deleteInstance,
		handleInstanceContextMenu,
		handleInstanceOption,
		handleInstanceIconSaved,
	}
}

export type LibraryState = ReturnType<typeof createLibraryState>

const libraryKey: InjectionKey<LibraryState> = Symbol('library')

export function provideLibrary(instances: Ref<GameInstance[]>) {
	const library = createLibraryState(instances)
	provide(libraryKey, library)
	return library
}

export function useLibrary() {
	const library = inject(libraryKey)

	if (!library) {
		throw new Error('useLibrary must be called within a library provider')
	}

	return library
}
