import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ManySelect from '../ManySelect.vue'

const meta = {
	title: 'Base/ManySelect',
	component: ManySelect,
} satisfies Meta<typeof ManySelect>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: { modelValue: [], options: [] },
	render: () => ({
		components: { ManySelect },
		setup() {
			const selected = ref<string[]>([])
			const options = ['Option 1', 'Option 2', 'Option 3', 'Option 4', 'Option 5']
			return { selected, options }
		},
		template: `
			<ManySelect v-model="selected" :options="options" showAlways>
				Select options ({{ selected.length }} selected)
			</ManySelect>
		`,
	}),
}

export const WithSearch: Story = {
	args: { modelValue: [], options: [] },
	render: () => ({
		components: { ManySelect },
		setup() {
			const selected = ref<string[]>(['Fabric', 'Forge'])
			const options = [
				'Fabric',
				'Forge',
				'Quilt',
				'NeoForge',
				'Paper',
				'Spigot',
				'Bukkit',
				'Sponge',
			]
			return { selected, options }
		},
		template: `
			<ManySelect v-model="selected" :options="options" search showAlways>
				Loaders ({{ selected.length }} selected)
			</ManySelect>
		`,
	}),
}

export const WithCustomDisplay: Story = {
	args: { modelValue: [], options: [] },
	render: () => ({
		components: { ManySelect },
		setup() {
			const selected = ref<{ id: string; name: string }[]>([])
			const options = [
				{ id: '1', name: 'Adventure' },
				{ id: '2', name: 'Decoration' },
				{ id: '3', name: 'Technology' },
				{ id: '4', name: 'Magic' },
			]
			const displayName = (opt: { id: string; name: string }) => opt.name
			return { selected, options, displayName }
		},
		template: `
			<ManySelect v-model="selected" :options="options" :displayName="displayName" showAlways>
				Categories
			</ManySelect>
		`,
	}),
}
