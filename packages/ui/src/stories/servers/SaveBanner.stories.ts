import type { Meta, StoryObj } from '@storybook/vue3-vite'

import SaveBanner from '../../components/servers/SaveBanner.vue'

const meta = {
	title: 'Servers/SaveBanner',
	component: SaveBanner,
	parameters: {
		layout: 'fullscreen',
	},
	args: {
		isVisible: true,
		isUpdating: false,
		restart: false,
		serverId: 'server_123',
		save: () => {},
		reset: () => {},
	},
} satisfies Meta<typeof SaveBanner>

export default meta

type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Saving: Story = {
	args: {
		isUpdating: true,
	},
}

export const WithRestart: Story = {
	args: {
		restart: true,
	},
}
