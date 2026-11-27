import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import SortMenu, { type SortField, type SortMode } from '../../components/base/SortMenu.vue'

const meta = {
	title: 'Base/SortMenu',
	component: SortMenu,
} satisfies Meta<typeof SortMenu>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { SortMenu },
		setup() {
			const fields: SortField[] = [
				{ value: 'published', label: 'Date published', kind: 'date' },
				{ value: 'downloads', label: 'Downloads', kind: 'numeric' },
				{ value: 'version', label: 'Version number', kind: 'alphabetical' },
			]
			const mode = ref<SortMode>({ field: 'published', direction: 'desc' })
			return { fields, mode }
		},
		template: /* html */ `
			<div class="flex justify-end">
				<SortMenu v-model="mode" :fields="fields" />
			</div>
		`,
	}),
}
