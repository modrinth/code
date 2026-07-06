import {
	AffiliateIcon,
	BoxIcon,
	CalendarIcon,
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

import AutoLink from '../../components/base/AutoLink.vue'
import ButtonStyled from '../../components/base/ButtonStyled.vue'
import FormattedTag from '../../components/base/FormattedTag.vue'
import JoinedButtons from '../../components/base/JoinedButtons.vue'
import PageHeader from '../../components/base/page-header/index.vue'
import PageHeaderMetadata from '../../components/base/page-header/metadata/index.vue'
import PageHeaderMetadataItem from '../../components/base/page-header/metadata/page-header-metadata-item.vue'
import PageHeaderMetadataNumberItem from '../../components/base/page-header/metadata/page-header-metadata-number-item.vue'
import PageHeaderMetadataTagsItem from '../../components/base/page-header/metadata/page-header-metadata-tags-item.vue'
import PageHeaderMetadataTimeItem from '../../components/base/page-header/metadata/page-header-metadata-time-item.vue'
import PageHeaderActions from '../../components/base/page-header/page-header-actions.vue'
import PageHeaderBadgeItem from '../../components/base/page-header/page-header-badge-item.vue'
import PageHeaderObjectAvatarLeading from '../../components/base/page-header/page-header-object-avatar-leading.vue'
import PageHeaderUserAvatarLeading from '../../components/base/page-header/page-header-user-avatar-leading.vue'
import TagItem from '../../components/base/TagItem.vue'
import TeleportOverflowMenu from '../../components/base/TeleportOverflowMenu.vue'
import LoaderIcon from '../../components/servers/icons/LoaderIcon.vue'
import ServerIcon from '../../components/servers/icons/ServerIcon.vue'

const noop = () => undefined
const categories = ['adventure', 'magic', 'technology']
const joinedDate = new Date(Date.now() - 1000 * 60 * 60 * 24 * 365 * 6)
const menuActions = [
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
]
const joinedActions = [
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
]

const pageHeaderIcons = {
	AffiliateIcon,
	BoxIcon,
	CalendarIcon,
	DownloadIcon,
	Gamepad2Icon,
	GlobeIcon,
	HeartIcon,
	LeftArrowIcon,
	LinkIcon,
	MoreVerticalIcon,
	PlayIcon,
	SettingsIcon,
	TimerIcon,
}

const pageHeaderComponents = {
	AutoLink,
	ButtonStyled,
	FormattedTag,
	JoinedButtons,
	PageHeader,
	PageHeaderActions,
	PageHeaderBadgeItem,
	PageHeaderMetadata,
	PageHeaderMetadataItem,
	PageHeaderMetadataNumberItem,
	PageHeaderMetadataTagsItem,
	PageHeaderMetadataTimeItem,
	PageHeaderObjectAvatarLeading,
	PageHeaderUserAvatarLeading,
	TagItem,
	TeleportOverflowMenu,
	...pageHeaderIcons,
}

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

export const ProjectHeader: Story = {
	render: () => ({
		components: pageHeaderComponents,
		setup() {
			return {
				...pageHeaderIcons,
				categories,
				menuActions,
				noop,
			}
		},
		template: `
			<PageHeader title="Fabric API" summary="Lightweight and modular API providing common hooks and intercompatibility measures for Fabric mods.">
				<template #leading>
					<PageHeaderObjectAvatarLeading src="" alt="Fabric API" size="96px" tint-by="fabric-api" />
				</template>

				<template #badges>
					<PageHeaderBadgeItem :icon="AffiliateIcon">
						Featured
					</PageHeaderBadgeItem>
				</template>

				<template #metadata>
					<PageHeaderMetadata>
						<PageHeaderMetadataNumberItem :icon="DownloadIcon" :value="128452395" label="downloads" />
						<PageHeaderMetadataNumberItem :icon="HeartIcon" :value="412300" label="followers" />
						<PageHeaderMetadataTagsItem>
							<TagItem v-for="category in categories" :key="category" :action="noop">
								<FormattedTag :tag="category" />
							</TagItem>
						</PageHeaderMetadataTagsItem>
					</PageHeaderMetadata>
				</template>

				<template #actions>
					<PageHeaderActions>
						<ButtonStyled color="brand" size="large">
							<button type="button" @click="noop">
								<DownloadIcon />
								Download
							</button>
						</ButtonStyled>
						<ButtonStyled circular size="large" type="transparent">
							<TeleportOverflowMenu :options="menuActions" aria-label="More actions">
								<MoreVerticalIcon />
							</TeleportOverflowMenu>
						</ButtonStyled>
					</PageHeaderActions>
				</template>
			</PageHeader>
		`,
	}),
}

export const CreatorHeader: Story = {
	render: () => ({
		components: pageHeaderComponents,
		setup() {
			return {
				...pageHeaderIcons,
				joinedDate,
				noop,
			}
		},
		template: `
			<PageHeader title="Prospector" summary="A Modrinth creator with a handful of popular projects.">
				<template #leading>
					<PageHeaderUserAvatarLeading src="" username="Prospector" />
				</template>

				<template #badges>
					<PageHeaderBadgeItem :icon="AffiliateIcon" class="!border-brand-highlight !bg-brand-highlight !text-brand">
						Affiliate
					</PageHeaderBadgeItem>
				</template>

				<template #metadata>
					<PageHeaderMetadata>
						<PageHeaderMetadataNumberItem :icon="BoxIcon" :value="12" label="projects" />
						<PageHeaderMetadataNumberItem :icon="DownloadIcon" :value="4200000" label="downloads" />
						<PageHeaderMetadataTimeItem :icon="CalendarIcon" :date="joinedDate" label="Joined" />
					</PageHeaderMetadata>
				</template>

				<template #actions>
					<PageHeaderActions>
						<ButtonStyled color="brand" size="large">
							<button type="button" @click="noop">
								<HeartIcon />
								Follow
							</button>
						</ButtonStyled>
					</PageHeaderActions>
				</template>
			</PageHeader>
		`,
	}),
}

export const AppInstanceHeader: Story = {
	render: () => ({
		components: {
			...pageHeaderComponents,
			LoaderIcon,
		},
		setup() {
			return {
				...pageHeaderIcons,
				LoaderIcon,
				menuActions,
				noop,
			}
		},
		template: `
			<PageHeader title="Create: Astral">
				<template #leading>
					<PageHeaderObjectAvatarLeading src="" alt="Create: Astral" size="64px" tint-by="create-astral" />
				</template>

				<template #metadata>
					<PageHeaderMetadata>
						<PageHeaderMetadataItem :icon="Gamepad2Icon" tooltip="Minecraft version">Minecraft 1.20.1</PageHeaderMetadataItem>
						<PageHeaderMetadataItem :icon="LoaderIcon" :icon-props="{ loader: 'Fabric' }" tooltip="Mod loader">
							Fabric 0.16.14
						</PageHeaderMetadataItem>
						<PageHeaderMetadataItem :icon="TimerIcon" tooltip="Total playtime">12 hours</PageHeaderMetadataItem>
					</PageHeaderMetadata>
				</template>

				<template #actions>
					<PageHeaderActions>
						<ButtonStyled color="brand" size="large">
							<button type="button" @click="noop">
								<PlayIcon />
								Play
							</button>
						</ButtonStyled>
						<ButtonStyled circular size="large">
							<button type="button" aria-label="Instance settings" @click="noop">
								<SettingsIcon />
							</button>
						</ButtonStyled>
						<ButtonStyled circular size="large" type="transparent">
							<TeleportOverflowMenu :options="menuActions" aria-label="More actions">
								<MoreVerticalIcon />
							</TeleportOverflowMenu>
						</ButtonStyled>
					</PageHeaderActions>
				</template>
			</PageHeader>
		`,
	}),
}

export const BrowseHeader: Story = {
	render: () => ({
		components: {
			...pageHeaderComponents,
			LoaderIcon,
		},
		setup() {
			return {
				...pageHeaderIcons,
				LoaderIcon,
				noop,
			}
		},
		template: `
			<PageHeader title="Survival SMP" :divider="false" :bottom-padding="false" main-class="items-center" title-class="leading-8" truncate-title>
				<template #leading>
					<ButtonStyled circular size="large">
						<button type="button" aria-label="Back to instance" @click="noop">
							<LeftArrowIcon />
						</button>
					</ButtonStyled>
					<PageHeaderObjectAvatarLeading src="" alt="Survival SMP" size="48px" tint-by="survival-smp" />
				</template>

				<template #metadata>
					<PageHeaderMetadata>
						<PageHeaderMetadataItem class="!text-primary">Installing content</PageHeaderMetadataItem>
						<PageHeaderMetadataItem class="!text-primary" :icon="Gamepad2Icon" tooltip="Minecraft version">
							Minecraft 1.20.1
						</PageHeaderMetadataItem>
						<PageHeaderMetadataItem class="!text-primary" :icon="LoaderIcon" :icon-props="{ loader: 'Fabric' }" tooltip="Mod loader">
							Fabric
						</PageHeaderMetadataItem>
					</PageHeaderMetadata>
				</template>
			</PageHeader>
		`,
	}),
}

export const ServerPanelRootHeader: Story = {
	render: () => ({
		components: {
			...pageHeaderComponents,
			ServerIcon,
		},
		setup() {
			return {
				...pageHeaderIcons,
				menuActions,
				noop,
				serverImage: undefined,
			}
		},
		template: `
			<PageHeader title="Survival SMP">
				<template #leading>
					<ServerIcon class="size-16 !rounded-xl" :image="serverImage" />
				</template>

				<template #metadata>
					<PageHeaderMetadata>
						<PageHeaderMetadataItem :icon="GlobeIcon" tooltip="Active instance">My World</PageHeaderMetadataItem>
						<PageHeaderMetadataItem :icon="LinkIcon" tooltip="Copy server address" :action="noop">
							play.modrinth.gg
						</PageHeaderMetadataItem>
					</PageHeaderMetadata>
				</template>

				<template #actions>
					<PageHeaderActions>
						<ButtonStyled color="brand" size="large">
							<button type="button" @click="noop">
								<PlayIcon />
								Start server
							</button>
						</ButtonStyled>
						<ButtonStyled circular size="large">
							<button type="button" aria-label="Server settings" @click="noop">
								<SettingsIcon />
							</button>
						</ButtonStyled>
					</PageHeaderActions>
				</template>
			</PageHeader>
		`,
	}),
}

export const ServerPanelInstanceHeader: Story = {
	render: () => ({
		components: {
			...pageHeaderComponents,
			LoaderIcon,
		},
		setup() {
			return {
				...pageHeaderIcons,
				joinedActions,
				LoaderIcon,
				noop,
			}
		},
		template: `
			<PageHeader title="My World">
				<template #leading>
					<ButtonStyled circular size="large">
						<button type="button" aria-label="All instances" @click="noop">
							<LeftArrowIcon />
						</button>
					</ButtonStyled>
				</template>

				<template #metadata>
					<PageHeaderMetadata>
						<PageHeaderMetadataItem :icon="Gamepad2Icon" tooltip="Minecraft version">Minecraft 1.20.1</PageHeaderMetadataItem>
						<PageHeaderMetadataItem :icon="LoaderIcon" :icon-props="{ loader: 'Fabric' }" tooltip="Mod loader">
							Fabric 0.19.2
						</PageHeaderMetadataItem>
						<PageHeaderMetadataItem :icon="TimerIcon" tooltip="Last activity">Last active 2 weeks ago</PageHeaderMetadataItem>
					</PageHeaderMetadata>
				</template>

				<template #actions>
					<PageHeaderActions>
						<ButtonStyled color="brand" size="large">
							<button type="button" @click="noop">
								<PlayIcon />
								Start instance
							</button>
						</ButtonStyled>
						<JoinedButtons :actions="joinedActions" color="red" size="large" />
						<ButtonStyled circular size="large">
							<button type="button" aria-label="Instance settings" @click="noop">
								<SettingsIcon />
							</button>
						</ButtonStyled>
					</PageHeaderActions>
				</template>
			</PageHeader>
		`,
	}),
}

export const CustomMetadata: Story = {
	render: () => ({
		components: pageHeaderComponents,
		setup() {
			return {
				...pageHeaderIcons,
			}
		},
		template: `
			<PageHeader
				title="Custom Metadata Project"
				summary="Custom metadata stays in markup so page-specific components can be composed without expanding PageHeader itself."
			>
				<template #leading>
					<PageHeaderObjectAvatarLeading src="" alt="Custom Metadata Project" size="96px" tint-by="custom-metadata-project" />
				</template>

				<template #metadata>
					<PageHeaderMetadata>
						<PageHeaderMetadataItem>
							<div class="flex flex-wrap items-center gap-3">
								<div class="flex items-center gap-2 font-semibold">
									<DownloadIcon class="h-6 w-6 text-secondary" />
									1.2M
								</div>
								<div class="flex items-center gap-2 font-semibold">
									<HeartIcon class="h-6 w-6 text-secondary" />
									50K
								</div>
							</div>
						</PageHeaderMetadataItem>
					</PageHeaderMetadata>
				</template>
			</PageHeader>
		`,
	}),
}
