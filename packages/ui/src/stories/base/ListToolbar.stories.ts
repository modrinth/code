import { RefreshCwIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import IconButton from '../../components/base/buttons/IconButton.vue'
import FilterPills from '../../components/base/FilterPills.vue'
import ListToolbar from '../../components/base/ListToolbar.vue'

const meta = {
	title: 'Base/ListToolbar',
	component: ListToolbar,
} satisfies Meta<typeof ListToolbar>

export default meta

const categories = [
	{
		key: 'channel',
		label: 'Channel',
		options: [
			{ value: 'release', label: 'Release' },
			{ value: 'beta', label: 'Beta' },
			{ value: 'alpha', label: 'Alpha' },
		],
	},
	{
		key: 'platform',
		label: 'Platform',
		options: [
			{ value: 'fabric', label: 'Fabric' },
			{ value: 'neoforge', label: 'NeoForge' },
			{ value: 'quilt', label: 'Quilt' },
		],
	},
]

export const Default: StoryObj = {
	render: () => ({
		components: { ListToolbar, IconButton, RefreshCwIcon },
		setup() {
			const search = ref('')
			const filters = ref<Record<string, string[]>>({ platform: ['fabric'] })
			const page = ref(1)
			return { search, filters, page, categories }
		},
		template: /* html */ `
			<ListToolbar
				v-model:search="search"
				v-model:filters="filters"
				:filter-categories="categories"
				search-placeholder="Search 120 versions..."
				:page="page"
				:page-count="6"
				@switch-page="page = $event"
			>
				<template #actions>
					<IconButton type="quiet" size="lg" label="Refresh">
						<RefreshCwIcon />
					</IconButton>
				</template>
			</ListToolbar>
		`,
	}),
}

export const CustomFilters: StoryObj = {
	render: () => ({
		components: { ListToolbar, FilterPills },
		setup() {
			const search = ref('')
			const selected = ref<string[]>([])
			const options = [
				{ id: 'online', label: 'Online' },
				{ id: 'offline', label: 'Offline' },
			]
			return { search, selected, options }
		},
		template: /* html */ `
			<ListToolbar v-model:search="search" search-placeholder="Search servers..." filters-shown>
				<template #filters>
					<FilterPills v-model="selected" :options="options" />
				</template>
			</ListToolbar>
		`,
	}),
}
