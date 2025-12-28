import type { Meta, StoryObj } from '@storybook/vue3-vite'

import SimpleBadge from '../../components/base/SimpleBadge.vue'

const meta = {
	title: 'Base/SimpleBadge',
	component: SimpleBadge,
} satisfies Meta<typeof SimpleBadge>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		formattedName: 'Badge Text',
	},
}
