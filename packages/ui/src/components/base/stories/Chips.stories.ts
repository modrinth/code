import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Chips from '../Chips.vue'

const meta = {
	title: 'Base/Chips',
	// for vue generic typed component
	// https://stackoverflow.com/questions/78037116/how-can-i-create-stories-for-generically-typed-vue-components
	component: Chips as Record<keyof typeof Chips, unknown>,
	tags: ['autodocs'],
	argTypes: {
		size: { control: 'select', options: ['standard', 'small'] },
		neverEmpty: { control: 'boolean' },
		capitalize: { control: 'boolean' },
	},
	args: {
		items: ['Option 1', 'Option 2', 'Option 3'],
		neverEmpty: true,
		capitalize: true,
		size: 'standard',
	},
} satisfies Meta<typeof Chips>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		items: ['Option 1', 'Option 2', 'Option 3'],
	},
}

export const Small: Story = {
	args: {
		items: ['Option 1', 'Option 2', 'Option 3'],
		size: 'small',
	},
}

export const AllowEmpty: Story = {
	args: {
		items: ['Option 1', 'Option 2', 'Option 3'],
		neverEmpty: false,
	},
}

export const NoCapitalize: Story = {
	args: {
		items: ['Option 1', 'Option 2', 'Option 3'],
		capitalize: false,
	},
}

export const ManyItems: Story = {
	args: {
		items: ['Alpha', 'Beta', 'Gamma', 'Delta', 'Epsilon', 'Zeta'],
	},
}
