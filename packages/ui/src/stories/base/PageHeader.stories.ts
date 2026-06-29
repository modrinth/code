import {
	AffiliateIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	GlobeIcon,
	HeartIcon,
	LeftArrowIcon,
	LinkIcon,
	MoreVerticalIcon,
	PlayIcon,
	SettingsIcon,
	SlashIcon,
	StopCircleIcon,
	TagCategoryGamepad2Icon as Gamepad2Icon,
	TimerIcon,
} from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import PageHeader from '../../components/base/PageHeader.vue'
import LoaderIcon from '../../components/servers/icons/LoaderIcon.vue'
import ServerIcon from '../../components/servers/icons/ServerIcon.vue'

const noop = () => undefined

const meta = {
	title: 'Base/PageHeader',
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
				menuActions: [
					{
						id: 'open-folder',
						label: 'Open folder',
						icon: GlobeIcon,
						action: noop,
					},
					{
						id: 'copy-id',
						label: 'Copy ID',
						icon: ClipboardCopyIcon,
						action: noop,
					},
				],
			},
		],
	},
}

export const CreatorHeader: Story = {
	args: {
		header: 'Prospector',
		summary: 'A Modrinth creator with a handful of popular projects.',
		leading: {
			type: 'avatar',
			src: null,
			alt: 'Prospector',
			avatarSize: '96px',
			circle: true,
		},
		badges: [
			{
				id: 'affiliate',
				label: 'Affiliate',
				icon: AffiliateIcon,
				class: 'border-brand-highlight bg-brand-highlight text-brand',
			},
		],
		metadata: [
			{
				id: 'projects',
				label: '12 projects',
				icon: Gamepad2Icon,
			},
			{
				id: 'downloads',
				label: '4.2M downloads',
				icon: DownloadIcon,
			},
			{
				id: 'followers',
				label: '82K followers',
				icon: HeartIcon,
			},
		],
		actions: [
			{
				id: 'follow',
				label: 'Follow',
				icon: HeartIcon,
				color: 'brand',
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
				id: 'stop',
				label: 'Stop instance',
				color: 'red',
				joinedActions: [
					{
						id: 'stop',
						label: 'Stop',
						icon: StopCircleIcon,
						action: noop,
					},
					{
						id: 'kill_server',
						label: 'Kill server',
						icon: SlashIcon,
						action: noop,
					},
				],
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

export const CustomMetadata: Story = {
	render: () => ({
		components: { PageHeader, DownloadIcon, HeartIcon },
		template: `
			<PageHeader
				header="Custom Metadata Project"
				summary="Custom metadata is reserved for rich stat rows that cannot be represented by icon and label data."
				:leading="{ type: 'avatar', src: null, alt: 'Custom Metadata Project', avatarSize: '96px' }"
				:metadata="[{ id: 'project-stats', type: 'custom', class: 'contents' }]"
			>
				<template #metadata-project-stats>
					<div class="flex flex-wrap items-center gap-3">
						<div class="flex items-center gap-2 font-semibold">
							<DownloadIcon class="h-6 w-6 text-secondary" />
							1.2M downloads
						</div>
						<div class="flex items-center gap-2 font-semibold">
							<HeartIcon class="h-6 w-6 text-secondary" />
							50K followers
						</div>
					</div>
				</template>
			</PageHeader>
		`,
	}),
}
