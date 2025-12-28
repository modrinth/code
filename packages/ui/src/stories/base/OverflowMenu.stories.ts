import { SettingsIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import OverflowMenu from '../../components/base/OverflowMenu.vue'

const meta = {
	title: 'Base/OverflowMenu',
	component: OverflowMenu,
	render: (args) => ({
		components: { OverflowMenu, SettingsIcon },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<OverflowMenu v-bind="args">
				<SettingsIcon />
				<template #edit>Edit</template>
				<template #delete>Delete</template>
				<template #share>Share</template>
			</OverflowMenu>
		`,
	}),
} satisfies Meta<typeof OverflowMenu>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		options: [
			{ id: 'edit', action: () => console.log('Edit clicked') },
			{ id: 'share', action: () => console.log('Share clicked') },
			{ divider: true },
			{ id: 'delete', action: () => console.log('Delete clicked'), color: 'danger' },
		],
	},
}
