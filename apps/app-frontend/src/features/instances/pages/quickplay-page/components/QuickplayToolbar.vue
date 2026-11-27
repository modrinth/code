<script setup lang="ts">
import { CompassIcon, PlusIcon, RefreshCwIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	type FilterPillOption,
	FilterPills,
	IconButton,
	ListToolbar,
	type ToolbarTab,
	ToolbarTabs,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import type { QuickplayTab } from '../types'

const props = defineProps<{
	worldCount: number
	filterOptions: FilterPillOption[]
	showServerActions: boolean
	refreshing: boolean
}>()

const activeTab = defineModel<QuickplayTab>('tab', { required: true })
const search = defineModel<string>('search', { required: true })
const filters = defineModel<string[]>('filters', { required: true })

const emit = defineEmits<{
	addServer: [event: MouseEvent]
	browseServers: []
	refresh: []
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	tabsLabel: {
		id: 'app.instance.worlds.tabs-label',
		defaultMessage: 'Quickplay types',
	},
	serversTab: {
		id: 'app.instance.worlds.tab.servers',
		defaultMessage: 'Servers',
	},
	worldsTab: {
		id: 'app.instance.worlds.tab.worlds',
		defaultMessage: 'Worlds',
	},
	searchServersLabel: {
		id: 'app.instance.worlds.search-servers-label',
		defaultMessage: 'Search servers',
	},
	searchWorldsLabel: {
		id: 'app.instance.worlds.search-worlds-label',
		defaultMessage: 'Search worlds',
	},
	searchServersPlaceholder: {
		id: 'app.instance.worlds.search-servers-placeholder',
		defaultMessage: 'Search {count, number} servers...',
	},
	searchWorldsPlaceholder: {
		id: 'app.instance.worlds.search-worlds-placeholder',
		defaultMessage: 'Search {count, number} worlds...',
	},
	addServer: {
		id: 'app.instance.worlds.add-server',
		defaultMessage: 'Add server',
	},
	browseServers: {
		id: 'app.instance.worlds.browse-servers',
		defaultMessage: 'Browse servers',
	},
	refreshing: {
		id: 'app.instance.worlds.refreshing',
		defaultMessage: 'Refreshing...',
	},
})

const tabs = computed<ToolbarTab<QuickplayTab>[]>(() => [
	{ value: 'servers', label: formatMessage(messages.serversTab) },
	{ value: 'worlds', label: formatMessage(messages.worldsTab) },
])

const isServersTab = computed(() => activeTab.value === 'servers')

const showFilterPills = computed(() => isServersTab.value && props.filterOptions.length > 0)

const searchLabel = computed(() =>
	formatMessage(isServersTab.value ? messages.searchServersLabel : messages.searchWorldsLabel),
)

const searchPlaceholder = computed(() =>
	formatMessage(
		isServersTab.value ? messages.searchServersPlaceholder : messages.searchWorldsPlaceholder,
		{ count: props.worldCount },
	),
)

const refreshLabel = computed(() =>
	formatMessage(props.refreshing ? messages.refreshing : commonMessages.refreshButton),
)
</script>

<template>
	<div class="flex flex-col gap-4">
		<div class="flex items-center justify-between gap-2">
			<ToolbarTabs v-model="activeTab" :tabs="tabs" :label="formatMessage(messages.tabsLabel)" />
			<div v-if="isServersTab && showServerActions" class="ml-auto flex gap-2">
				<Button type="outlined" size="lg" @click="emit('browseServers')">
					<CompassIcon aria-hidden="true" />
					{{ formatMessage(messages.browseServers) }}
				</Button>
				<Button type="colored" color="brand" size="lg" @click="emit('addServer', $event)">
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.addServer) }}
				</Button>
			</div>
		</div>
		<ListToolbar
			v-if="worldCount > 0"
			v-model:search="search"
			:search-label="searchLabel"
			:search-placeholder="searchPlaceholder"
			:filters-shown="showFilterPills"
		>
			<template #actions>
				<IconButton
					v-tooltip="refreshLabel"
					type="quiet"
					size="lg"
					:label="refreshLabel"
					:disabled="refreshing"
					@click="emit('refresh')"
				>
					<RefreshCwIcon :class="{ 'animate-spin': refreshing }" aria-hidden="true" />
				</IconButton>
			</template>
			<template #filters>
				<FilterPills v-model="filters" :options="filterOptions">
					<template #all>
						{{ formatMessage(commonMessages.allProjectType) }}
					</template>
				</FilterPills>
			</template>
		</ListToolbar>
	</div>
</template>
