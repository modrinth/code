import type { Meta, StoryObj } from '@storybook/vue3-vite'

import MedalServerListing from '../../components/servers/marketing/MedalServerListing.vue'

const baseMedalServer = {
	server_id: '8759b459-6f9e-49e0-aa70-dfab5b2abb2f',
	name: 'Medal Trial Server',
	status: 'available',
	game: 'Minecraft',
	mc_version: '1.21.5',
	loader: 'Fabric',
	loader_version: '0.16.14',
	net: {
		ip: '203.0.113.42',
		port: 25565,
		domain: 'play',
	},
	medal_expires: new Date(Date.now() + 3 * 24 * 60 * 60 * 1000).toISOString(),
} as const

const meta = {
	title: 'Servers/MedalServerListing',
	component: MedalServerListing,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 920px;"><story /></div>',
		}),
	],
} satisfies Meta<typeof MedalServerListing>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		...baseMedalServer,
	},
}

export const ConfiguringNewServer: Story = {
	args: {
		...baseMedalServer,
		name: 'New Medal Server',
		flows: { intro: true },
	},
}

export const SuspendedUpgrading: Story = {
	args: {
		...baseMedalServer,
		name: 'Upgrading Server',
		status: 'suspended',
		suspension_reason: 'upgrading',
	},
}

export const SuspendedCancelled: Story = {
	args: {
		...baseMedalServer,
		name: 'Expired Trial Server',
		status: 'suspended',
		suspension_reason: 'cancelled',
	},
}

export const SuspendedWithReason: Story = {
	args: {
		...baseMedalServer,
		name: 'Suspended Server',
		status: 'suspended',
		suspension_reason: 'moderated',
	},
}

export const SuspendedGeneric: Story = {
	args: {
		...baseMedalServer,
		name: 'Suspended Server',
		status: 'suspended',
	},
}
