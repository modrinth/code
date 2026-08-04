import { PlayIcon, SettingsIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ServerInstanceManageHeader from '../../components/servers/server-header/ServerInstanceManageHeader.vue'

const meta = {
	title: 'Servers/ServerInstanceManageHeader',
	component: ServerInstanceManageHeader,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 920px;"><story /></div>',
		}),
	],
} satisfies Meta<typeof ServerInstanceManageHeader>

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
		actions: [
			{
				id: 'start',
				label: 'Start instance',
				icon: PlayIcon,
				color: 'brand',
				onClick: () => undefined,
			},
			{
				id: 'settings',
				label: 'Instance settings',
				icon: SettingsIcon,
				labelHidden: true,
				tooltip: 'Instance settings',
				onClick: () => undefined,
			},
		],
	},
}
