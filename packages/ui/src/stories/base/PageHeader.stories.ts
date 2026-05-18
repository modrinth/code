import {
	GlobeIcon,
	LeftArrowIcon,
	LinkIcon,
	MoreVerticalIcon,
	PlayIcon,
	SettingsIcon,
	TagCategoryGamepad2Icon as Gamepad2Icon,
	TimerIcon,
} from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import PageHeader from '../../components/base/PageHeader.vue'
import LoaderIcon from '../../components/servers/icons/LoaderIcon.vue'
import ServerIcon from '../../components/servers/icons/ServerIcon.vue'

const noop = () => undefined

const meta = {
	title: 'Base/PageHeader Prototype',
	component: PageHeader,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story) => ({
			components: { story },
			template: '<div class="w-full"><story /></div>',
		}),
	],
} satisfies Meta<typeof PageHeader>

export default meta
type Story = StoryObj<typeof meta>

export const AppInstanceHeader: Story = {
	args: {
		header: 'Create: Astral',
		leading: {
			type: 'avatar',
			src: null,
			alt: 'Create: Astral',
			tintBy: '/Users/calum/Library/Application Support/com.modrinth.theseus/profiles/astral',
		},
		metadata: [
			{
				id: 'game-version',
				label: '1.20.1',
				icon: Gamepad2Icon,
				tooltip: 'Minecraft version',
			},
			{
				id: 'loader',
				label: 'Fabric 0.16.14',
				icon: LoaderIcon,
				iconProps: { loader: 'Fabric' },
				tooltip: 'Mod loader',
			},
			{
				id: 'playtime',
				label: '12 hours',
				icon: TimerIcon,
				tooltip: 'Total playtime',
			},
		],
		actions: [
			{
				id: 'play',
				label: 'Play',
				icon: PlayIcon,
				color: 'brand',
				onClick: noop,
			},
			{
				id: 'settings',
				label: 'Instance settings',
				icon: SettingsIcon,
				labelHidden: true,
				tooltip: 'Instance settings',
				onClick: noop,
			},
			{
				id: 'more',
				label: 'More actions',
				icon: MoreVerticalIcon,
				labelHidden: true,
				type: 'transparent',
				tooltip: 'More actions',
				onClick: noop,
			},
		],
	},
}

export const BrowseHeader: Story = {
	args: {
		header: 'Survival SMP',
		leading: [
			{
				id: 'back',
				type: 'button',
				icon: LeftArrowIcon,
				to: '/instance/my-world',
				ariaLabel: 'Back to instance',
				tooltip: 'Back to instance',
				wrapperClass: 'flex size-12 shrink-0 items-center justify-center',
			},
			{
				id: 'target',
				type: 'avatar',
				src: null,
				alt: 'Survival SMP',
				avatarSize: '48px',
				tintBy: 'survival-smp',
			},
		],
		metadata: [
			{
				id: 'heading',
				label: 'Installing content',
				class: '!text-primary',
			},
			{
				id: 'game-version',
				label: '1.20.1',
				icon: Gamepad2Icon,
				tooltip: 'Minecraft version',
				class: '!text-primary',
			},
			{
				id: 'loader',
				label: 'Fabric',
				icon: LoaderIcon,
				iconProps: { loader: 'Fabric' },
				tooltip: 'Mod loader',
				class: '!text-primary',
			},
		],
		divider: false,
		bottomPadding: false,
		mainClass: 'items-center',
		titleClass: 'leading-8',
		truncateTitle: true,
	},
}

export const ServerPanelRootHeader: Story = {
	args: {
		header: 'Survival SMP',
		leading: {
			type: 'component',
			component: ServerIcon,
			componentProps: { image: undefined },
			class: 'size-15 !rounded-xl',
		},
		metadata: [
			{
				id: 'active-world',
				label: 'My World',
				icon: GlobeIcon,
				tooltip: 'Active instance',
			},
			{
				id: 'address',
				label: 'play.modrinth.gg',
				icon: LinkIcon,
				tooltip: 'Copy server address',
				onClick: noop,
			},
		],
		actions: [
			{
				id: 'start',
				label: 'Start server',
				icon: PlayIcon,
				color: 'brand',
				onClick: noop,
			},
			{
				id: 'settings',
				label: 'Server settings',
				icon: SettingsIcon,
				labelHidden: true,
				tooltip: 'Server settings',
				onClick: noop,
			},
		],
	},
}

export const ServerPanelInstanceHeader: Story = {
	args: {
		header: 'My World',
		leading: {
			type: 'button',
			icon: LeftArrowIcon,
			to: '/hosting/manage/demo-server/instances',
			ariaLabel: 'All instances',
			tooltip: 'All instances',
		},
		metadata: [
			{
				id: 'game-version',
				label: '1.20.1',
				icon: Gamepad2Icon,
				tooltip: 'Minecraft version',
			},
			{
				id: 'loader',
				label: 'Fabric 0.19.2',
				icon: LoaderIcon,
				iconProps: { loader: 'Fabric' },
				tooltip: 'Mod loader',
			},
			{
				id: 'last-active',
				label: 'Last active 2 weeks ago',
				icon: TimerIcon,
				tooltip: 'Last activity',
			},
		],
		actions: [
			{
				id: 'start',
				label: 'Start instance',
				icon: PlayIcon,
				color: 'brand',
				onClick: noop,
			},
			{
				id: 'settings',
				label: 'Instance settings',
				icon: SettingsIcon,
				labelHidden: true,
				tooltip: 'Instance settings',
				onClick: noop,
			},
		],
	},
}
