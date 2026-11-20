import { defineMessages, type FilterPillOption, useVIntl } from '@modrinth/ui'
import { computed, type MaybeRefOrGetter, type Ref, ref, toValue, watch } from 'vue'

import { getWorldIdentifier, worldNameMatchesQuery } from '@/features/worlds/world-utils'
import type { ServerWorld, SingleplayerWorld, World } from '@/platform/app-lib/worlds/types'

import type { QuickplayTab, ServerAvailability } from '../types'

const messages = defineMessages({
	vanillaFilter: {
		id: 'app.instance.worlds.filter-vanilla',
		defaultMessage: 'Vanilla',
	},
	moddedFilter: {
		id: 'app.instance.worlds.filter-modded',
		defaultMessage: 'Modded',
	},
	onlineFilter: {
		id: 'app.instance.worlds.filter-online',
		defaultMessage: 'Online',
	},
	offlineFilter: {
		id: 'app.instance.worlds.filter-offline',
		defaultMessage: 'Offline',
	},
})

function isModded(server: ServerWorld) {
	return server.content_kind === 'modpack'
}

export function useQuickplayFilters(options: {
	serverWorlds: Ref<ServerWorld[]>
	singleplayerWorlds: Ref<SingleplayerWorld[]>
	getServerAvailability: (server: ServerWorld) => ServerAvailability
	highlightedWorld: MaybeRefOrGetter<string | undefined>
	loaded: MaybeRefOrGetter<boolean>
}) {
	const { serverWorlds, singleplayerWorlds, getServerAvailability } = options
	const { formatMessage } = useVIntl()

	const activeTab = ref<QuickplayTab>('servers')
	const searchQuery = ref('')
	const selectedFilters = ref<string[]>([])

	const activeWorlds = computed<World[]>(() =>
		activeTab.value === 'servers' ? serverWorlds.value : singleplayerWorlds.value,
	)

	const filterOptions = computed(() => {
		const filters: FilterPillOption[] = []
		const servers = serverWorlds.value

		if (servers.some((server) => !isModded(server)) && servers.some(isModded)) {
			filters.push({ id: 'vanilla', label: formatMessage(messages.vanillaFilter) })
			filters.push({ id: 'modded', label: formatMessage(messages.moddedFilter) })
		}

		const availabilities = new Set(servers.map(getServerAvailability))
		if (availabilities.has('online') && availabilities.has('offline')) {
			filters.push({ id: 'online', label: formatMessage(messages.onlineFilter) })
			filters.push({ id: 'offline', label: formatMessage(messages.offlineFilter) })
		}

		return filters
	})

	watch(filterOptions, (filters) => {
		const validIds = new Set(filters.map((filter) => filter.id))
		const cleaned = selectedFilters.value.filter((id) => validIds.has(id))
		if (cleaned.length !== selectedFilters.value.length) selectedFilters.value = cleaned
	})

	watch(activeTab, () => {
		searchQuery.value = ''
	})

	function passesServerFilters(server: ServerWorld) {
		const typeFilters = selectedFilters.value.filter((id) => id === 'vanilla' || id === 'modded')
		const statusFilters = selectedFilters.value.filter((id) => id === 'online' || id === 'offline')

		const passesType =
			typeFilters.length === 0 || typeFilters.includes(isModded(server) ? 'modded' : 'vanilla')
		const availability = getServerAvailability(server)
		const passesStatus =
			statusFilters.length === 0 ||
			(availability !== 'pending' && statusFilters.includes(availability))

		return passesType && passesStatus
	}

	const filteredWorlds = computed(() =>
		activeWorlds.value.filter((world) => {
			if (searchQuery.value && !worldNameMatchesQuery(world.name, searchQuery.value)) return false
			return world.type !== 'server' || passesServerFilters(world)
		}),
	)

	let initialTabChosen = false

	watch(
		() => toValue(options.loaded),
		(loaded) => {
			if (!loaded || initialTabChosen) return
			initialTabChosen = true
			const highlighted = toValue(options.highlightedWorld)
			const highlightsWorld = singleplayerWorlds.value.some(
				(world) => getWorldIdentifier(world) === highlighted,
			)
			const onlyWorlds = serverWorlds.value.length === 0 && singleplayerWorlds.value.length > 0
			if (highlightsWorld || onlyWorlds) activeTab.value = 'worlds'
		},
		{ immediate: true },
	)

	return {
		activeTab,
		searchQuery,
		selectedFilters,
		filterOptions,
		activeWorlds,
		filteredWorlds,
	}
}
