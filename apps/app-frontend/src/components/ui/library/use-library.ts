import type { Labrinth } from '@modrinth/api-client'
import { formatLoader, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useEventListener, useStorage } from '@vueuse/core'
import dayjs from 'dayjs'
import { computed, inject, type InjectionKey, provide, type Ref, ref, watchEffect } from 'vue'

import { get_project_v3_many } from '@/helpers/cache.js'
import { toError } from '@/helpers/errors'
import { install_duplicate_instance } from '@/helpers/install'
import { edit, remove } from '@/helpers/instance'
import {
	create_group as createInstanceGroup,
	delete_group as deleteInstanceGroup,
	type InstanceGroupDefinition,
	list_groups as listInstanceGroups,
	rename_group as renameInstanceGroup,
} from '@/helpers/instance-groups'
import type { GameInstance } from '@/helpers/types'

export const librarySortOptions = [
	'Manual',
	'Name',
	'Last played',
	'Hours played',
	'Date created',
	'Date modified',
] as const

export const libraryGroupOptions = [
	{ value: 'Group', label: 'Custom group' },
	{ value: 'Loader', label: 'Loader' },
	{ value: 'Game version', label: 'Game version' },
	{ value: 'None', label: 'No grouping' },
] as const

export type LibrarySort = (typeof librarySortOptions)[number]
export type LibraryGroupBy = (typeof libraryGroupOptions)[number]['value']
export type LibraryFilters = Record<'instanceType' | 'gameVersion' | 'loader', string[]>

export type InstanceGroup = {
	id: string
	key: string
	instances: GameInstance[]
}

export type ActiveInstanceGroupDrag = {
	instanceIds: string[]
	primaryInstanceId: string
	fromGroup: string | null
}

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

type ContextMenuSelection = {
	option: string
	item: InstanceCard
}

function createLibraryState(instances: Ref<GameInstance[]>) {
	const { handleError } = injectNotificationManager()
	const { formatMessage } = useVIntl()

	const search = ref('')
	const filters = ref<LibraryFilters>({
		instanceType: [],
		gameVersion: [],
		loader: [],
	})
	const serverProjectIds = ref(new Set<string>())
	const libraryGroups = ref<InstanceGroupDefinition[]>([])
	const groupIdsByName = ref(new Map<string, string>())
	const isNewGroupModalOpen = ref(false)
	const newGroupName = ref('')
	const newGroupSearch = ref('')
	const selectedNewGroupInstanceIds = ref(new Set<string>())
	/** Instance-level selection shared by every group representation of an instance. */
	const selectedLibraryInstanceIds = ref(new Set<string>())
	const isLibraryInstanceSelectionActive = computed(() => selectedLibraryInstanceIds.value.size > 0)
	const creatingGroup = ref(false)
	const activeInstanceGroupDrag = ref<ActiveInstanceGroupDrag | null>(null)
	const activeDraggedInstanceIds = computed(
		() => new Set(activeInstanceGroupDrag.value?.instanceIds ?? []),
	)
	const instanceGroupDragTarget = ref<string | null>(null)
	const instanceGroupDragPointer = ref({ x: 0, y: 0 })
	const isAddingInstanceToGroup = ref(false)
	const instanceOptions = ref<InstanceContextMenu | null>(null)
	const currentDeleteInstanceId = ref<string | null>(null)
	const currentContextGroupName = ref<string | null>(null)
	const confirmDeleteModal = ref<ConfirmDeleteModal | null>(null)

	const displayState = useStorage<{
		group: LibraryGroupBy
		sortBy: LibrarySort
		collapsedGroups: string[]
	}>(
		'Instances-grid-display-state',
		{
			group: 'Group',
			sortBy: 'Name',
			collapsedGroups: [],
		},
		localStorage,
		{ mergeDefaults: true },
	)

	if (!librarySortOptions.includes(displayState.value.sortBy)) {
		displayState.value.sortBy = 'Name'
	}

	const linkedInstances = computed(() => instances.value.filter((instance) => instance.link))
	const collapsedSectionKeys = computed(() => new Set(displayState.value.collapsedGroups))
	const groupNames = computed(
		() =>
			new Set(
				[
					...libraryGroups.value.map((group) => group.name),
					...instances.value.flatMap((instance) => instance.groups),
				]
					.map((group) => group.trim())
					.filter((group) => group && group.toLowerCase() !== 'none'),
			),
	)
	const existingGroupNames = computed(
		() => new Set(['none', ...Array.from(groupNames.value, (group) => group.toLowerCase())]),
	)
	const normalizedNewGroupName = computed(() => newGroupName.value.trim().substring(0, 32))
	const newGroupNameExists = computed(() =>
		existingGroupNames.value.has(normalizedNewGroupName.value.toLowerCase()),
	)
	const newGroupInstances = computed(() => {
		const query = newGroupSearch.value.trim().toLowerCase()

		return instances.value
			.filter((instance) => !query || instance.name.toLowerCase().includes(query))
			.slice()
			.sort((a, b) => {
				const groupedDifference = Number(a.groups.length > 0) - Number(b.groups.length > 0)
				if (groupedDifference !== 0) return groupedDifference

				return a.name.localeCompare(b.name)
			})
	})
	const canCreateGroup = computed(
		() =>
			normalizedNewGroupName.value.length > 0 && !newGroupNameExists.value && !creatingGroup.value,
	)

	const refreshGroups = async () => {
		try {
			const groups = await listInstanceGroups()
			libraryGroups.value = groups
			groupIdsByName.value = new Map([
				...groupIdsByName.value,
				...groups.map((group) => [group.name, group.id] as const),
			])
		} catch (error) {
			handleError(toError(error))
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
			case 'Manual':
				break
			case 'Name':
				visibleInstances.sort((a, b) => a.name.localeCompare(b.name))
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

		const groupedInstances = new Map<string, GameInstance[]>()
		const addToGroup = (key: string, instance: GameInstance) => {
			const group = groupedInstances.get(key) ?? []
			group.push(instance)
			groupedInstances.set(key, group)
		}

		for (const instance of visibleInstances) {
			switch (displayState.value.group) {
				case 'Loader':
					addToGroup(formatLoader(formatMessage, instance.loader), instance)
					break
				case 'Game version':
					addToGroup(instance.game_version, instance)
					break
				case 'Group':
					for (const group of instance.groups.length > 0 ? instance.groups : ['None']) {
						addToGroup(group, instance)
					}
					break
				case 'None':
					addToGroup('None', instance)
					break
			}
		}

		const resolveGroupId = (groupName: string) =>
			groupIdsByName.value.get(groupName) ?? `group-name:${groupName}`

		if (displayState.value.group === 'Group') {
			if (!groupedInstances.has('None')) {
				groupedInstances.set('None', [])
			}

			const populatedGroupIds = new Set(
				instances.value.flatMap((instance) =>
					instance.groups.map((groupName) => resolveGroupId(groupName)),
				),
			)

			for (const group of libraryGroups.value) {
				if (!populatedGroupIds.has(group.id) && !groupedInstances.has(group.name)) {
					groupedInstances.set(group.name, [])
				}
			}
		}

		const groups = Array.from(groupedInstances, ([key, groupInstances]) => ({
			id:
				displayState.value.group === 'Group'
					? key === 'None'
						? 'group:none'
						: resolveGroupId(key)
					: `${displayState.value.group}:${key}`,
			key,
			instances: groupInstances,
		}))

		if (displayState.value.sortBy === 'Name') {
			groups.sort((a, b) => a.key.localeCompare(b.key))
		}

		if (displayState.value.group === 'Game version') {
			groups.sort((a, b) => a.key.localeCompare(b.key, undefined, { numeric: true }))
		}

		if (displayState.value.group === 'Group') {
			groups.sort((a, b) => {
				if (a.key === 'None') return 1
				if (b.key === 'None') return -1
				return a.key.localeCompare(b.key)
			})
		}

		return groups
	})

	const getSectionKey = (sectionName: string) => `${displayState.value.group}:${sectionName}`

	const isSectionCollapsed = (sectionName: string) =>
		collapsedSectionKeys.value.has(getSectionKey(sectionName))

	const setSectionCollapsed = (sectionName: string, collapsed: boolean) => {
		const sectionKey = getSectionKey(sectionName)
		const collapsedSections = new Set(displayState.value.collapsedGroups)

		if (collapsed) {
			collapsedSections.add(sectionKey)
		} else {
			collapsedSections.delete(sectionKey)
		}

		displayState.value.collapsedGroups = [...collapsedSections]
	}

	const normalizeInstanceGroupName = (groupName: string) =>
		groupName === 'None' ? null : groupName

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
		groupName: string,
		pointer?: { x: number; y: number },
		altKey = false,
	) => {
		if (displayState.value.group !== 'Group') return

		const instanceIds = selectedLibraryInstanceIds.value.has(instanceId)
			? [
					instanceId,
					...[...selectedLibraryInstanceIds.value].filter(
						(selectedId) => selectedId !== instanceId,
					),
				].filter((selectedId) => instances.value.some((instance) => instance.id === selectedId))
			: [instanceId]

		activeInstanceGroupDrag.value = {
			instanceIds,
			primaryInstanceId: instanceId,
			fromGroup: normalizeInstanceGroupName(groupName),
		}
		updateInstanceGroupDrag(pointer, altKey)
	}

	const finishInstanceGroupDrag = () => {
		activeInstanceGroupDrag.value = null
		instanceGroupDragTarget.value = null
		isAddingInstanceToGroup.value = false
	}

	const setInstanceGroupDragTarget = (groupName: string | null) => {
		instanceGroupDragTarget.value = groupName
	}

	const getInstanceGroupDropState = (groupName: string) => {
		const drag = activeInstanceGroupDrag.value
		const draggedInstances = drag
			? instances.value.filter((instance) => drag.instanceIds.includes(instance.id))
			: []
		const toGroup = normalizeInstanceGroupName(groupName)
		const operation = isAddingInstanceToGroup.value ? 'add' : 'move'
		const canAddToGroup = operation !== 'add' || toGroup !== null
		const hasChanges = draggedInstances.some((instance) => {
			if (operation === 'add') {
				return toGroup !== null && !instance.groups.includes(toGroup)
			}

			return toGroup === null
				? instance.groups.length > 0
				: instance.groups.length !== 1 || instance.groups[0] !== toGroup
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
			if (!dropState.canDrop || dropState.operation !== 'add') return

			const count = activeInstanceGroupDrag.value?.instanceIds.length ?? 0
			return count > 1 ? `Duplicate ${count} instances to group` : 'Duplicate into group'
		}

		if (isAddingInstanceToGroup.value) return 'Duplicate to group'
		return
	})

	const moveDraggedInstancesToGroup = async (groupName: string, addToGroup = false) => {
		const drag = activeInstanceGroupDrag.value
		const toGroup = normalizeInstanceGroupName(groupName)
		if (!drag) return false

		const dropState = getInstanceGroupDropState(groupName)
		if (!dropState.canDrop) return false

		const shouldAdd = addToGroup && toGroup !== null
		const draggedInstances = instances.value.filter((instance) =>
			drag.instanceIds.includes(instance.id),
		)
		const results = await Promise.allSettled(
			draggedInstances.map((instance) => {
				const nextGroups = shouldAdd ? [...instance.groups] : toGroup ? [toGroup] : []
				if (shouldAdd && toGroup && !nextGroups.includes(toGroup)) {
					nextGroups.push(toGroup)
				}

				return edit(instance.id, { groups: nextGroups })
			}),
		)
		let movedInstanceCount = 0

		for (const result of results) {
			if (result.status === 'rejected') {
				handleError(toError(result.reason))
			} else {
				movedInstanceCount++
			}
		}

		return movedInstanceCount > 0
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

	const openNewGroupModal = () => {
		let groupNumber = groupNames.value.size + 1
		while (existingGroupNames.value.has(`group ${groupNumber}`.toLowerCase())) {
			groupNumber++
		}

		newGroupName.value = `Group ${groupNumber}`
		newGroupSearch.value = ''
		selectedNewGroupInstanceIds.value = new Set()
		isNewGroupModalOpen.value = true
	}

	const closeNewGroupModal = () => {
		isNewGroupModalOpen.value = false
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
		selectedLibraryInstanceIds.value = new Set()
	}

	const setSelectedLibraryInstanceIds = (instanceIds: Iterable<string>) => {
		selectedLibraryInstanceIds.value = new Set(instanceIds)
	}

	const toggleLibraryInstanceSelection = (instanceId: string) => {
		const selectedIds = new Set(selectedLibraryInstanceIds.value)

		if (selectedIds.has(instanceId)) {
			selectedIds.delete(instanceId)
		} else {
			selectedIds.add(instanceId)
		}

		selectedLibraryInstanceIds.value = selectedIds
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
				...libraryGroups.value.filter((existingGroup) => existingGroup.id !== group.id),
				group,
			]
			groupIdsByName.value = new Map(groupIdsByName.value).set(group.name, group.id)

			await Promise.all(
				instances.value
					.filter((instance) => selectedNewGroupInstanceIds.value.has(instance.id))
					.map((instance) =>
						edit(instance.id, {
							groups: [group.name],
						}),
					),
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

	const deleteGroup = async (groupName: string) => {
		try {
			await deleteInstanceGroup(groupName)
			libraryGroups.value = libraryGroups.value.filter((group) => group.name !== groupName)
			const nextGroupIdsByName = new Map(groupIdsByName.value)
			nextGroupIdsByName.delete(groupName)
			groupIdsByName.value = nextGroupIdsByName
			return true
		} catch (error) {
			handleError(toError(error))
			await refreshGroups()
			return false
		}
	}

	const isValidGroupName = (groupName: string, currentGroupName: string) => {
		const normalizedGroupName = groupName.trim()
		if (
			normalizedGroupName.length === 0 ||
			normalizedGroupName.length > 32 ||
			normalizedGroupName.toLowerCase() === 'none'
		) {
			return false
		}

		return !Array.from(groupNames.value).some(
			(existingGroupName) =>
				existingGroupName.toLowerCase() === normalizedGroupName.toLowerCase() &&
				existingGroupName !== currentGroupName,
		)
	}

	const renameGroup = async (groupId: string, oldName: string, newName: string) => {
		const normalizedNewName = newName.trim()
		if (oldName === normalizedNewName) return true
		if (!isValidGroupName(normalizedNewName, oldName)) return false

		const previousNewNameId = groupIdsByName.value.get(normalizedNewName)
		groupIdsByName.value = new Map(groupIdsByName.value).set(normalizedNewName, groupId)

		try {
			const renamedGroup = await renameInstanceGroup(oldName, normalizedNewName)
			libraryGroups.value = [
				...libraryGroups.value.filter((group) => group.id !== groupId),
				renamedGroup,
			]
			groupIdsByName.value = new Map(groupIdsByName.value).set(renamedGroup.name, renamedGroup.id)

			const oldSectionKey = getSectionKey(oldName)
			const newSectionKey = getSectionKey(renamedGroup.name)
			if (collapsedSectionKeys.value.has(oldSectionKey)) {
				displayState.value.collapsedGroups = displayState.value.collapsedGroups.map((sectionKey) =>
					sectionKey === oldSectionKey ? newSectionKey : sectionKey,
				)
			}

			if (currentContextGroupName.value === oldName) {
				currentContextGroupName.value = renamedGroup.name
			}

			return true
		} catch (error) {
			const nextGroupIdsByName = new Map(groupIdsByName.value)
			if (previousNewNameId) {
				nextGroupIdsByName.set(normalizedNewName, previousNewNameId)
			} else {
				nextGroupIdsByName.delete(normalizedNewName)
			}
			groupIdsByName.value = nextGroupIdsByName
			handleError(toError(error))
			await refreshGroups()
			return false
		}
	}

	const deleteInstance = async () => {
		if (!currentDeleteInstanceId.value) return

		await remove(currentDeleteInstanceId.value).catch((error) => handleError(toError(error)))
		currentDeleteInstanceId.value = null
	}

	const duplicateInstance = async (instanceId: string) => {
		await install_duplicate_instance(instanceId).catch((error) => handleError(toError(error)))
	}

	const handleInstanceContextMenu = (
		event: MouseEvent,
		item: InstanceCard,
		instanceGroupName: string,
	) => {
		currentContextGroupName.value =
			displayState.value.group === 'Group' && instanceGroupName !== 'None'
				? instanceGroupName
				: null

		const baseOptions = [
			...(item.instance.quarantined ? [] : [{ name: 'add_content' }, { type: 'divider' }]),
			{ name: 'edit' },
			{ name: 'duplicate' },
			{ name: 'open' },
			{ name: 'copy' },
			...(currentContextGroupName.value
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
			case 'edit':
				await item.seeInstance()
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
				if (currentContextGroupName.value) {
					const groupName = currentContextGroupName.value
					await edit(item.instance.id, {
						groups: item.instance.groups.filter((group) => group !== groupName),
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
		search,
		filters,
		displayState,
		instanceGroups,
		isNewGroupModalOpen,
		newGroupName,
		newGroupSearch,
		selectedNewGroupInstanceIds,
		selectedLibraryInstanceIds,
		isLibraryInstanceSelectionActive,
		activeInstanceGroupDrag,
		activeDraggedInstanceIds,
		instanceGroupDragTarget,
		instanceGroupDragPointer,
		instanceGroupDragStatus,
		isAddingInstanceToGroup,
		creatingGroup,
		newGroupNameExists,
		newGroupInstances,
		canCreateGroup,
		instanceOptions,
		confirmDeleteModal,
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
		toggleNewGroupInstance,
		clearLibraryInstanceSelection,
		setSelectedLibraryInstanceIds,
		toggleLibraryInstanceSelection,
		createGroup,
		deleteGroup,
		isValidGroupName,
		renameGroup,
		deleteInstance,
		handleInstanceContextMenu,
		handleInstanceOption,
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
