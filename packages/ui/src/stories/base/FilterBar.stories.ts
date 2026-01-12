import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import FilterBar from '../../components/base/FilterBar.vue'

const meta = {
	title: 'Base/FilterBar',
	component: FilterBar,
} satisfies Meta<typeof FilterBar>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { FilterBar },
		setup() {
			const selected = ref<string[]>([])
			const options = [
				{ id: 'active', message: { id: 'filter.active', defaultMessage: 'Active' } },
				{ id: 'archived', message: { id: 'filter.archived', defaultMessage: 'Archived' } },
				{ id: 'draft', message: { id: 'filter.draft', defaultMessage: 'Draft' } },
			]
			return { selected, options }
		},
		template: `
			<FilterBar v-model="selected" :options="options" showAllOptions />
		`,
	}),
}

export const WithSelection: StoryObj = {
	render: () => ({
		components: { FilterBar },
		setup() {
			const selected = ref<string[]>(['mods', 'plugins'])
			const options = [
				{ id: 'mods', message: { id: 'filter.mods', defaultMessage: 'Mods' } },
				{ id: 'plugins', message: { id: 'filter.plugins', defaultMessage: 'Plugins' } },
				{
					id: 'resourcepacks',
					message: { id: 'filter.resourcepacks', defaultMessage: 'Resource Packs' },
				},
				{ id: 'modpacks', message: { id: 'filter.modpacks', defaultMessage: 'Modpacks' } },
			]
			return { selected, options }
		},
		template: `
			<FilterBar v-model="selected" :options="options" showAllOptions />
		`,
	}),
}
