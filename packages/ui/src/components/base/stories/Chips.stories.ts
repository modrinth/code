import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Chips from '../Chips.vue'

const meta = {
	title: 'Base/Chips',
	// @ts-ignore - error comes from generically typed component
	component: Chips,
	tags: ['autodocs'],
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

export const NeverEmpty: Story = {
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
