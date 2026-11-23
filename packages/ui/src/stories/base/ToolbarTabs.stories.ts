import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ToolbarTabs from '../../components/base/ToolbarTabs.vue'

const meta = {
	title: 'Base/ToolbarTabs',
	component: ToolbarTabs,
} satisfies Meta<typeof ToolbarTabs>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { ToolbarTabs },
		setup() {
			const value = ref('mod')
			const tabs = [
				{ value: 'mod', label: 'Mods' },
				{ value: 'resourcepack', label: 'Resource Packs' },
				{ value: 'datapack', label: 'Data Packs' },
				{ value: 'shader', label: 'Shaders' },
			]

			return { value, tabs }
		},
		template: /* html */ `
			<ToolbarTabs v-model="value" :tabs="tabs" label="Project types" />
		`,
	}),
}

export const FullWidth: StoryObj = {
	render: () => ({
		components: { ToolbarTabs },
		setup() {
			const value = ref('asc')
			const tabs = [
				{ value: 'asc', label: 'A-Z' },
				{ value: 'desc', label: 'Z-A' },
			]

			return { value, tabs }
		},
		template: /* html */ `
			<div class="w-64">
				<ToolbarTabs v-model="value" :tabs="tabs" label="Sort direction" full-width />
			</div>
		`,
	}),
}
