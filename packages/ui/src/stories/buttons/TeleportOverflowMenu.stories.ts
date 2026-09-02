import {
	ArrowLeftRightIcon,
	DownloadIcon,
	ExternalIcon,
	MoreVerticalIcon,
	PlusIcon,
	SettingsIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import TeleportOverflowMenu from '../../components/base/buttons/TeleportOverflowMenu.vue'
import type { ButtonMenuOption } from '../../components/base/buttons/types'

const options: ButtonMenuOption[] = [
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
		type: 'base',
		size: 'md',
		placement: 'bottom-end',
		disabled: false,
		hoverable: false,
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
		type: 'colored',
	},
}

export const OutlinedTrigger: Story = {
	args: {
		type: 'outlined',
	},
}

export const QuietTrigger: Story = {
	args: {
		type: 'quiet',
	},
}

export const Hoverable: Story = {
	args: {
		hoverable: true,
	},
}

const optionsWithSubmenu: ButtonMenuOption[] = [
	...options,
	{ type: 'divider' },
	{
		id: 'switch-account',
		label: 'Switch account',
		icon: ArrowLeftRightIcon,
		type: 'submenu',
		options: [
			{
				id: 'account-jai',
				label: 'Jai',
				selected: true,
				action: () => undefined,
				trailingAction: {
					label: 'Remove Jai',
					icon: XIcon,
					color: 'red',
					action: () => undefined,
				},
			},
			{
				id: 'account-prospector',
				label: 'Prospector',
				selected: false,
				action: () => undefined,
				trailingAction: {
					label: 'Remove Prospector',
					icon: XIcon,
					color: 'red',
					action: () => undefined,
				},
			},
			{ type: 'divider' },
			{ id: 'add-account', label: 'Add account', icon: PlusIcon, action: () => undefined },
		],
	},
]

export const WithSubmenu: Story = {
	args: {
		options: optionsWithSubmenu,
	},
}
