import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Combobox from '../../components/base/Combobox.vue'

const meta = {
	title: 'Base/Combobox',
	// @ts-ignore - generic component
	component: Combobox,
} satisfies Meta<typeof Combobox>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		options: [
			{ value: '1', label: 'Option 1' },
			{ value: '2', label: 'Option 2' },
			{ value: '3', label: 'Option 3' },
		],
		triggerText: 'Select an option',
	},
}

export const Searchable: Story = {
	args: {
		options: [
			{ value: '1', label: 'Minecraft' },
			{ value: '2', label: 'Fabric' },
			{ value: '3', label: 'Forge' },
			{ value: '4', label: 'NeoForge' },
			{ value: '5', label: 'Quilt' },
		],
		triggerText: 'Select a loader',
		searchable: true,
		searchPlaceholder: 'Search loaders...',
	},
}

export const Disabled: Story = {
	args: {
		options: [{ value: '1', label: 'Option 1' }],
		triggerText: 'Disabled',
		disabled: true,
	},
}
