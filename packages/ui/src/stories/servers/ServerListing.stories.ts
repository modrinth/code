import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ServerListing from '../../components/servers/ServerListing.vue'

const baseServer = {
	server_id: '8759b459-6f9e-49e0-aa70-dfab5b2abb2f',
	name: 'Survival SMP',
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
} as const

const pendingChange = {
	planSize: 'Performance 8 GB',
	cpu: 4,
	cpuBurst: 6,
	ramGb: 8,
	storageGb: 80,
	date: new Date(Date.now() + 5 * 24 * 60 * 60 * 1000).toISOString(),
	verb: 'Upgrade',
} as const

const meta = {
	title: 'Servers/ServerListing',
	component: ServerListing,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 920px;"><story /></div>',
		}),
	],
} satisfies Meta<typeof ServerListing>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		...baseServer,
	},
}

export const ConfiguringNewServer: Story = {
	args: {
		...baseServer,
		name: 'Fresh Vanilla World',
		status: 'installing',
		flows: { intro: true },
		loader: 'Vanilla',
		loader_version: null,
	},
}

export const WithPendingChange: Story = {
	args: {
		...baseServer,
		name: 'Competitive UHC',
		pendingChange,
	},
}

export const SuspendedUpgrading: Story = {
	args: {
		...baseServer,
		name: 'Creative Build Team',
		status: 'suspended',
		suspension_reason: 'upgrading',
	},
}

export const SuspendedCancelled: Story = {
	args: {
		...baseServer,
		name: 'Old Event Server',
		status: 'suspended',
		suspension_reason: 'cancelled',
	},
}

export const SuspendedWithReason: Story = {
	args: {
		...baseServer,
		name: 'Minigames Network',
		status: 'suspended',
		suspension_reason: 'paymentfailed',
	},
}

export const SuspendedGeneric: Story = {
	args: {
		...baseServer,
		name: 'Archive Server',
		status: 'suspended',
	},
}
