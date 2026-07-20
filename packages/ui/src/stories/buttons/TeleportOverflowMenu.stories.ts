import {
	DownloadIcon,
	ExternalIcon,
	MoreVerticalIcon,
	SettingsIcon,
	TrashIcon,
} from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import TeleportOverflowMenu from '../../components/base/buttons/TeleportOverflowMenu.vue'
import type { OverflowMenuOption } from '../../components/base/buttons/types'

const options: OverflowMenuOption[] = [
	{
		id: 'download',
		label: 'Download',
		icon: DownloadIcon,
		action: () => undefined,
	},
	{
		id: 'settings',
		label: 'Project settings',
		icon: SettingsIcon,
		type: 'link',
		to: '/settings',
	},
	{
		id: 'website',
		label: 'Open website',
		icon: ExternalIcon,
		type: 'link',
		href: 'https://modrinth.com',
		target: '_blank',
	},
	{
		id: 'unavailable',
		label: 'Unavailable action',
		disabled: true,
		tooltip: 'This action is currently unavailable',
		action: () => undefined,
	},
	{ type: 'divider' },
	{
		id: 'delete',
		label: 'Delete project',
		icon: TrashIcon,
		tone: 'red',
		action: () => undefined,
	},
]

const meta = {
	title: 'Buttons/Teleport Overflow Menu',
	component: TeleportOverflowMenu,
	args: {
		label: 'More actions',
		options,
		variant: 'base',
		size: 'default',
		placement: 'bottom-end',
		disabled: false,
	},
	render: (args) => ({
		components: { MoreVerticalIcon, TeleportOverflowMenu },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<TeleportOverflowMenu v-bind="args">
				<MoreVerticalIcon />
			</TeleportOverflowMenu>
		`,
	}),
} satisfies Meta<typeof TeleportOverflowMenu>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const ColoredTrigger: Story = {
	args: {
		variant: 'colored',
	},
}

export const OutlinedTrigger: Story = {
	args: {
		variant: 'outlined',
	},
}

export const QuietTrigger: Story = {
	args: {
		variant: 'quiet',
	},
}
