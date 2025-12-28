import { SettingsIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Button from '../../components/base/Button.vue'
import PopoutMenu from '../../components/base/PopoutMenu.vue'

const meta = {
	title: 'Base/PopoutMenu',
	component: PopoutMenu,
	render: (args) => ({
		components: { PopoutMenu, Button, SettingsIcon },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<PopoutMenu v-bind="args">
				<SettingsIcon />
				<template #menu>
					<Button transparent>Option 1</Button>
					<Button transparent>Option 2</Button>
					<Button transparent>Option 3</Button>
				</template>
			</PopoutMenu>
		`,
	}),
} satisfies Meta<typeof PopoutMenu>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithTooltip: Story = {
	args: {
		tooltip: 'Click for more options',
	},
}
