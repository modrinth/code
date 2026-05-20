import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ServerListEmpty from '../../components/servers/server-list-empty/ServerListEmpty.vue'

const meta = {
	title: 'Servers/ServerListEmpty',
	component: ServerListEmpty,
	parameters: {
		layout: 'centered',
	},
} satisfies Meta<typeof ServerListEmpty>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
