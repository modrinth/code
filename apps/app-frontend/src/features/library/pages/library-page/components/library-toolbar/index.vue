<script setup lang="ts">
import { PlusIcon, SquarePlusIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	ListToolbar,
	type ToolbarTab,
	ToolbarTabs,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { injectInstanceCreation } from '@/features/instances/instance-creation-context'

import { injectLibrarySelection, injectLibraryView } from '../../library-context'
import type { LibraryTab } from '../../types'
import GroupMenu from './group-menu.vue'
import NewGroupModal from './new-group-modal.vue'
import SortMenu from './sort-menu.vue'
import { useLibraryFilterCategories } from './use-library-filter-categories'

const { search, filters, activeTab, tabInstances } = injectLibraryView()
const { selectedInstanceIds } = injectLibrarySelection()
const instanceCreation = injectInstanceCreation()
const newGroupModal = ref<InstanceType<typeof NewGroupModal>>()
const filterCategories = useLibraryFilterCategories()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	tabsLabel: { id: 'app.library.tabs.label', defaultMessage: 'Library sections' },
	installedTab: { id: 'app.library.tabs.installed', defaultMessage: 'Installed' },
	availableTab: { id: 'app.library.tabs.available', defaultMessage: 'Available' },
	searchLabel: { id: 'app.library.search.label', defaultMessage: 'Search instances' },
	searchPlaceholder: {
		id: 'app.library.search.instances-placeholder',
		defaultMessage: 'Search {count, plural, one {# instance} other {# instances}}...',
	},
	newGroup: { id: 'app.library.group.new', defaultMessage: 'New group' },
	newInstance: { id: 'app.library.instance.new', defaultMessage: 'New instance' },
})

const tabs = computed<ToolbarTab<LibraryTab>[]>(() => [
	{ value: 'installed', label: formatMessage(messages.installedTab) },
	{ value: 'available', label: formatMessage(messages.availableTab) },
])

const searchPlaceholder = computed(() =>
	formatMessage(messages.searchPlaceholder, { count: tabInstances.value.length }),
)

function openNewGroup(event: MouseEvent) {
	newGroupModal.value?.show(selectedInstanceIds.value, event)
}
</script>

<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-wrap items-center justify-between gap-2">
			<ToolbarTabs v-model="activeTab" :tabs="tabs" :label="formatMessage(messages.tabsLabel)" />
			<div class="ml-auto flex gap-2">
				<Button
					v-if="activeTab === 'installed' && tabInstances.length > 0"
					type="outlined"
					size="lg"
					@click="openNewGroup"
				>
					<SquarePlusIcon aria-hidden="true" />
					{{ formatMessage(messages.newGroup) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					size="lg"
					@click="instanceCreation.showCreationModal()"
				>
					<PlusIcon aria-hidden="true" />
					{{ formatMessage(messages.newInstance) }}
				</Button>
			</div>
		</div>
		<ListToolbar
			v-if="tabInstances.length > 0"
			v-model:search="search"
			v-model:filters="filters"
			:search-label="formatMessage(messages.searchLabel)"
			:search-placeholder="searchPlaceholder"
			:filter-categories="filterCategories"
		>
			<template #controls>
				<GroupMenu />
				<SortMenu />
			</template>
		</ListToolbar>
	</div>
	<NewGroupModal ref="newGroupModal" />
</template>
