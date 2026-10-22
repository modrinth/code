import { CalendarClockIcon, CpuIcon, DatabaseIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ServerStatCard from '../../components/servers/ServerStatCard.vue'

const meta = {
	title: 'Servers/ServerStatCard',
	component: ServerStatCard,
	decorators: [() => ({ template: '<div class="max-w-sm"><story /></div>' })],
} satisfies Meta<typeof ServerStatCard>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		title: 'CPU',
		icon: CpuIcon,
		value: '12.50%',
	},
}

export const WithSecondary: Story = {
	args: {
		title: 'Memory',
		icon: DatabaseIcon,
		value: '1.2 GiB',
		secondary: '/ 4 GiB',
	},
}

export const CustomContent: Story = {
	args: {
		title: 'Auto backups',
		icon: CalendarClockIcon,
	},
	render: (args) => ({
		components: { ServerStatCard },
		setup: () => ({ args }),
		template: `
			<ServerStatCard v-bind="args">
				<template #value>8<span class="ml-1 text-2xl font-semibold">/ 10 slots</span></template>
				<span class="text-secondary">Every 24h</span>
			</ServerStatCard>
		`,
	}),
}
