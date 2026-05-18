import { SettingsIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import WorldManageHeader from '../../components/servers/server-header/WorldManageHeader.vue'

const meta = {
	title: 'Servers/WorldManageHeader',
	component: WorldManageHeader,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 920px;"><story /></div>',
		}),
	],
} satisfies Meta<typeof WorldManageHeader>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		name: 'My World',
		gameVersion: '1.20.1',
		loader: 'fabric',
		loaderVersion: '0.19.2',
		lastActive: 'Last active 2 weeks ago',
		backHref: '/hosting/manage/demo-server/instances',
		backLabel: 'All instances',
	},
	render: (args) => ({
		components: { ButtonStyled, SettingsIcon, WorldManageHeader },
		setup() {
			return { args }
		},
		template: `
			<WorldManageHeader v-bind="args">
				<template #actions>
					<ButtonStyled size="large" circular>
						<button type="button" aria-label="Instance settings">
							<SettingsIcon aria-hidden="true" />
						</button>
					</ButtonStyled>
				</template>
			</WorldManageHeader>
		`,
	}),
}
