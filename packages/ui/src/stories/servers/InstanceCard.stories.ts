import type { Meta, StoryObj } from '@storybook/vue3-vite'

import InstanceCard from '../../components/servers/instances/InstanceCard.vue'

const meta = {
	title: 'Servers/InstanceCard',
	component: InstanceCard,
	parameters: {
		layout: 'padded',
	},
	render: (args) => ({
		components: { InstanceCard },
		setup() {
			return { args }
		},
		template: '<div style="max-width: 360px"><InstanceCard v-bind="args" /></div>',
	}),
} satisfies Meta<typeof InstanceCard>

export default meta

type Story = StoryObj<typeof meta>

export const Active: Story = {
	args: {
		world: {
			type: 'world',
			id: 'demo-world',
			name: 'Cobbletown',
			active: true,
			gameVersion: '1.20.4',
			loaderLabel: 'Fabric 0.16.6',
			linkedModpack: {
				name: 'Cobblemon Official',
				iconUrl: null,
				link: null,
			},
			installedContentCount: 47,
			lastActiveAt: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
			createdAt: new Date(Date.now() - 197 * 24 * 60 * 60 * 1000).toISOString(),
		},
	},
}

export const Empty: Story = {
	args: {
		world: {
			type: 'empty',
			id: 'empty-instance-slot',
			name: 'Instance #2',
		},
	},
}
