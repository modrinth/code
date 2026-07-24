import { formatLoader, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useStorage } from '@vueuse/core'
import dayjs from 'dayjs'
import {
	computed,
	inject,
	type InjectionKey,
	provide,
	type Ref,
	ref,
	watchEffect,
} from 'vue'

import { get_project_v3_many } from '@/helpers/cache.js'
import { install_duplicate_instance } from '@/helpers/install'
import { remove } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

export const libraryFilterOptions = [
	'All instances',
	'Modpacks',
	'Servers',
	'Custom',
] as const

export const librarySortOptions = [
	'Name',
	'Last played',
	'Date created',
	'Date modified',
	'Game version',
] as const

export const libraryGroupOptions = ['Group', 'Loader', 'Game version', 'None'] as const

export type LibrarySort = (typeof librarySortOptions)[number]
export type LibraryGroupBy = (typeof libraryGroupOptions)[number]
export type LibraryFilter = (typeof libraryFilterOptions)[number]

export type InstanceGroup = {
	key: string
	instances: GameInstance[]
}

type InstanceCard = {
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
	const activeFilter = ref<LibraryFilter>('All instances')
	const serverProjectIds = ref(new Set<string>())
	const isNewGroupModalOpen = ref(false)
	const instanceOptions = ref<InstanceContextMenu | null>(null)
	const instanceComponents = ref<InstanceCard[]>([])
	const currentDeleteInstanceId = ref<string | null>(null)
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

	const linkedInstances = computed(() => instances.value.filter((instance) => instance.link))
	const collapsedSectionKeys = computed(() => new Set(displayState.value.collapsedGroups))

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
			const projects = await get_project_v3_many(projectIds, 'must_revalidate')
			serverProjectIds.value = new Set(
				projects
					.filter((project) => project?.minecraft_server != null)
					.map((project) => project.id),
			)
		} catch {
			serverProjectIds.value = new Set()
		}
	})

	const filteredInstances = computed(() => {
		switch (activeFilter.value) {
			case 'Modpacks':
				return linkedInstances.value.filter(
					(instance) => !serverProjectIds.value.has(instance.link?.project_id ?? ''),
				)
			case 'Servers':
				return linkedInstances.value.filter((instance) =>
					serverProjectIds.value.has(instance.link?.project_id ?? ''),
				)
			case 'Custom':
				return instances.value.filter((instance) => !instance.link)
			default:
				return instances.value
		}
	})

	const instanceGroups = computed<InstanceGroup[]>(() => {
		const visibleInstances = filteredInstances.value.filter((instance) =>
			instance.name.toLowerCase().includes(search.value.toLowerCase()),
		)

		switch (displayState.value.sortBy) {
			case 'Name':
				visibleInstances.sort((a, b) => a.name.localeCompare(b.name))
				break
			case 'Game version':
				visibleInstances.sort((a, b) =>
					a.game_version.localeCompare(b.game_version, undefined, { numeric: true }),
				)
				break
			case 'Last played':
				visibleInstances.sort((a, b) =>
					dayjs(b.last_played ?? 0).diff(dayjs(a.last_played ?? 0)),
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

		const groups = Array.from(groupedInstances, ([key, groupInstances]) => ({
			key,
			instances: groupInstances,
		}))

		if (displayState.value.sortBy === 'Name') {
			groups.sort((a, b) => {
				if (a.key === 'None') return -1
				if (b.key === 'None') return 1
				return a.key.localeCompare(b.key)
			})
		}

		if (displayState.value.group === 'Game version') {
			groups.sort((a, b) => a.key.localeCompare(b.key, undefined, { numeric: true }))
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

	const openNewGroupModal = () => {
		isNewGroupModalOpen.value = true
	}

	const closeNewGroupModal = () => {
		isNewGroupModalOpen.value = false
	}

	const deleteInstance = async () => {
		if (!currentDeleteInstanceId.value) return

		await remove(currentDeleteInstanceId.value).catch(handleError)
		currentDeleteInstanceId.value = null
	}

	const duplicateInstance = async (instanceId: string) => {
		await install_duplicate_instance(instanceId).catch(handleError)
	}

	const handleInstanceContextMenu = (event: MouseEvent, instanceId: string) => {
		const item = instanceComponents.value.find(
			(instanceComponent) => instanceComponent.instance.id === instanceId,
		)
		if (!item) return

		const baseOptions = [
			...(item.instance.quarantined ? [] : [{ name: 'add_content' }, { type: 'divider' }]),
			{ name: 'edit' },
			{ name: 'duplicate' },
			{ name: 'open' },
			{ name: 'copy' },
			{ type: 'divider' },
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
			case 'delete':
				currentDeleteInstanceId.value = item.instance.id
				confirmDeleteModal.value?.show()
				break
		}
	}

	return {
		search,
		activeFilter,
		displayState,
		instanceGroups,
		isNewGroupModalOpen,
		instanceOptions,
		instanceComponents,
		confirmDeleteModal,
		isSectionCollapsed,
		setSectionCollapsed,
		openNewGroupModal,
		closeNewGroupModal,
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
