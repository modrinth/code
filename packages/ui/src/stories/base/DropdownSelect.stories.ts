import type { Meta, StoryObj } from '@storybook/vue3-vite'

import DropdownSelect from '../../components/base/DropdownSelect.vue'

const meta = {
	title: 'Base/DropdownSelect',
	component: DropdownSelect,
} satisfies Meta<typeof DropdownSelect>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		name: 'dropdown',
		options: ['Option 1', 'Option 2', 'Option 3'],
		modelValue: 'Option 1',
	},
}

export const ManyOptions: Story = {
	args: {
		name: 'sort',
		options: ['Relevance', 'Downloads', 'Follows', 'Newest', 'Updated'],
		modelValue: 'Relevance',
	},
}

export const Disabled: Story = {
	args: {
		name: 'disabled',
		options: ['Option 1', 'Option 2', 'Option 3'],
		modelValue: 'Option 1',
		disabled: true,
	},
}
