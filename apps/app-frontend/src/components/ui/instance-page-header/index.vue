<template>
	<PageHeader :title="instance.name">
		<template #leading>
			<Avatar :src="iconSrc" :alt="instance.name" size="64px" :tint-by="instance.id" />
		</template>

		<template v-if="instance.shared_instance" #badges>
			<PageHeaderBadgeItem
				:icon="UnknownIcon"
				:tooltip="sharedInstanceTooltip"
				aria-label="Shared instance information"
				class="!border-blue !bg-highlight-blue !text-blue"
			>
				Shared
			</PageHeaderBadgeItem>
		</template>

		<template #metadata>
			<div v-if="isServerInstance" class="flex flex-wrap items-center gap-2">
				<InstanceHeaderServerMetadata
					:loading-server-ping="loadingServerPing"
					:players-online="playersOnline"
					:status-online="statusOnline"
					:recent-plays="recentPlays"
					:ping="ping"
					:minecraft-server="minecraftServer"
					:linked-project-v3="linkedProjectV3"
					:instance-id="instance.id"
				/>
				<PageHeaderMetadataItem
					v-if="sharedInstanceManager"
					:action="sharedInstanceManagerAction"
				>
					{{ sharedInstanceManagerLabel }}
					<Avatar
						:src="sharedInstanceManager.avatarUrl"
						:alt="sharedInstanceManager.name"
						:tint-by="sharedInstanceManager.tintBy"
						size="24px"
						:circle="sharedInstanceManager.type === 'user'"
						no-shadow
					/>
					<span class="min-w-0 truncate">{{ sharedInstanceManager.name }}</span>
				</PageHeaderMetadataItem>
			</div>
			<PageHeaderMetadata v-else>
				<PageHeaderMetadataItem :icon="Gamepad2Icon" tooltip="Minecraft version">
					Minecraft {{ instance.game_version }}
				</PageHeaderMetadataItem>
				<PageHeaderMetadataItem
					:icon="ServerLoaderIcon"
					:icon-props="{ loader: loaderDisplayName }"
					tooltip="Mod loader"
				>
					{{ loaderLabel }}
				</PageHeaderMetadataItem>
				<PageHeaderMetadataItem
					v-if="showInstancePlayTime"
					:icon="TimerIcon"
					tooltip="Total playtime"
				>
					{{ playtimeLabel }}
				</PageHeaderMetadataItem>
				<PageHeaderMetadataItem
					v-if="sharedInstanceManager"
					:action="sharedInstanceManagerAction"
				>
					{{ sharedInstanceManagerLabel }}
					<Avatar
						:src="sharedInstanceManager.avatarUrl"
						:alt="sharedInstanceManager.name"
						:tint-by="sharedInstanceManager.tintBy"
						size="24px"
						:circle="sharedInstanceManager.type === 'user'"
						no-shadow
					/>
					<span class="min-w-0 truncate">{{ sharedInstanceManager.name }}</span>
				</PageHeaderMetadataItem>
			</PageHeaderMetadata>
		</template>

		<template #actions>
			<PageHeaderActions>
				<ButtonStyled v-if="isInstalling" color="brand" size="large">
					<button type="button" disabled>
						{{ formatMessage(commonMessages.installingLabel) }}
					</button>
				</ButtonStyled>
				<ButtonStyled v-else-if="instance.install_stage !== 'installed'" color="brand" size="large">
					<button type="button" @click="emit('repair')">
						<DownloadIcon />
						{{ formatMessage(messages.repair) }}
					</button>
				</ButtonStyled>
				<ButtonStyled v-else-if="playing" color="red" size="large">
					<button type="button" :disabled="stopping" @click="emit('stop')">
						<StopCircleIcon />
						{{
							stopping ? formatMessage(messages.stopping) : formatMessage(commonMessages.stopButton)
						}}
					</button>
				</ButtonStyled>
				<JoinedButtons
					v-else-if="!loading && isServerInstance"
					:actions="serverPlayActions"
					color="brand"
					size="large"
				/>
				<ButtonStyled v-else-if="!loading" color="brand" size="large">
					<button type="button" @click="emit('play')">
						<PlayIcon />
						{{ formatMessage(commonMessages.playButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled v-else color="brand" size="large">
					<button type="button" disabled>{{ formatMessage(messages.starting) }}</button>
				</ButtonStyled>

				<ButtonStyled circular size="large">
					<button
						v-tooltip="formatMessage(messages.instanceSettings)"
						type="button"
						:aria-label="formatMessage(messages.instanceSettings)"
						@click="emit('settings')"
					>
						<SettingsIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled circular size="large" type="transparent">
					<TeleportOverflowMenu
						:options="moreActions"
						:tooltip="formatMessage(messages.moreActions)"
						:aria-label="formatMessage(messages.moreActions)"
					>
						<MoreVerticalIcon />
					</TeleportOverflowMenu>
				</ButtonStyled>
			</PageHeaderActions>
		</template>
	</PageHeader>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	DownloadIcon,
	ExternalIcon,
	FolderOpenIcon,
	MoreVerticalIcon,
	PackageIcon,
	PlayIcon,
	SettingsIcon,
	StopCircleIcon,
	TagCategoryGamepad2Icon as Gamepad2Icon,
	TimerIcon,
	UnknownIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	formatLoaderLabel,
	type JoinedButtonAction,
	JoinedButtons,
	LoaderIcon as ServerLoaderIcon,
	PageHeader,
	PageHeaderActions,
	PageHeaderBadgeItem,
	PageHeaderMetadata,
	PageHeaderMetadataItem,
	type ServerLoader,
	TeleportOverflowMenu,
	type TeleportOverflowMenuItem,
	useVIntl,
} from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed } from 'vue'

import type { GameInstance } from '@/helpers/types'

import InstanceHeaderServerMetadata from './instance-page-header-server-metadata.vue'

const messages = defineMessages({
	createShortcut: {
		id: 'instance.action.create-shortcut',
		defaultMessage: 'Create shortcut',
	},
	exportModpack: {
		id: 'instance.action.export-modpack',
		defaultMessage: 'Export modpack',
	},
	instanceSettings: {
		id: 'instance.action.settings',
		defaultMessage: 'Instance settings',
	},
	launchInstance: {
		id: 'instance.action.launch-instance',
		defaultMessage: 'Launch instance',
	},
	moreActions: {
		id: 'instance.action.more-actions',
		defaultMessage: 'More actions',
	},
	neverPlayed: {
		id: 'instance.playtime.never-played',
		defaultMessage: 'Never played',
	},
	openFolder: {
		id: 'instance.action.open-folder',
		defaultMessage: 'Open folder',
	},
	repair: {
		id: 'instance.action.repair',
		defaultMessage: 'Repair',
	},
	starting: {
		id: 'instance.action.starting',
		defaultMessage: 'Starting...',
	},
	stopping: {
		id: 'instance.action.stopping',
		defaultMessage: 'Stopping...',
	},
	sharedInstanceTooltip: {
		id: 'instance.shared-instance.tooltip',
		defaultMessage: "This instance's content is being managed by someone else.",
	},
	sharedInstanceOwnerTooltip: {
		id: 'instance.shared-instance.owner-tooltip',
		defaultMessage: "This instance's content is being shared to other users.",
	},
})

const props = withDefaults(
	defineProps<{
		instance: GameInstance
		iconSrc?: string | null
		isServerInstance?: boolean
		showInstancePlayTime?: boolean
		timePlayed?: number
		playing?: boolean
		loading?: boolean
		stopping?: boolean
		loadingServerPing?: boolean
		playersOnline?: number
		statusOnline?: boolean
		recentPlays?: number
		ping?: number
		minecraftServer?: Labrinth.Projects.v3.Project['minecraft_server']
		linkedProjectV3?: Labrinth.Projects.v3.Project
		sharedInstanceManager?: {
			type: 'user' | 'server'
			name: string
			avatarUrl?: string
			tintBy: string
		} | null
	}>(),
	{
		iconSrc: null,
		isServerInstance: false,
		showInstancePlayTime: false,
		timePlayed: 0,
		playing: false,
		loading: false,
		stopping: false,
		loadingServerPing: false,
		playersOnline: undefined,
		statusOnline: false,
		recentPlays: undefined,
		ping: undefined,
		minecraftServer: undefined,
		linkedProjectV3: undefined,
		sharedInstanceManager: null,
	},
)

const emit = defineEmits<{
	repair: []
	stop: []
	play: []
	playServer: []
	settings: []
	openFolder: []
	export: []
	createShortcut: []
}>()

const installingStages = [
	'installing',
	'pack_installing',
	'pack_installed',
	'not_installed',
	'minecraft_installing',
]

const { formatMessage } = useVIntl()

const isInstalling = computed(() => installingStages.includes(props.instance.install_stage))
const loaderDisplayName = computed(() => formatLoaderLabel(props.instance.loader) as ServerLoader)
const loaderLabel = computed(() =>
	[loaderDisplayName.value, props.instance.loader_version].filter(Boolean).join(' '),
)
const sharedInstanceTooltip = computed(() =>
	formatMessage(
		props.instance.shared_instance?.role === 'owner'
			? messages.sharedInstanceOwnerTooltip
			: messages.sharedInstanceTooltip,
	),
)
const sharedInstanceManagerLabel = computed(() =>
	props.sharedInstanceManager?.type === 'server' ? 'Linked to' : 'Managed by',
)
const sharedInstanceManagerAction = computed(() => {
	const manager = props.sharedInstanceManager
	if (manager?.type !== 'user') return undefined
	return () => openUrl(`https://modrinth.com/user/${encodeURIComponent(manager.name)}`)
})
const playtimeLabel = computed(() => {
	if (props.timePlayed <= 0) return formatMessage(messages.neverPlayed)

	const hours = Math.floor(props.timePlayed / 3600)
	if (hours >= 1) {
		return `${hours} hour${hours > 1 ? 's' : ''}`
	}

	const minutes = Math.floor(props.timePlayed / 60)
	if (minutes >= 1) {
		return `${minutes} minute${minutes > 1 ? 's' : ''}`
	}

	const seconds = Math.floor(props.timePlayed)
	return `${seconds} second${seconds > 1 ? 's' : ''}`
})
const serverPlayActions = computed<JoinedButtonAction[]>(() => [
	{
		id: 'join_server',
		label: formatMessage(commonMessages.playButton),
		icon: PlayIcon,
		action: () => emit('playServer'),
	},
	{
		id: 'launch_instance',
		label: formatMessage(messages.launchInstance),
		icon: PlayIcon,
		action: () => emit('play'),
	},
])
const moreActions = computed<TeleportOverflowMenuItem[]>(() => [
	{
		id: 'open-folder',
		label: formatMessage(messages.openFolder),
		icon: FolderOpenIcon,
		action: () => emit('openFolder'),
	},
	{
		id: 'export-mrpack',
		label: formatMessage(messages.exportModpack),
		icon: PackageIcon,
		action: () => emit('export'),
	},
	{
		id: 'create-shortcut',
		label: formatMessage(messages.createShortcut),
		icon: ExternalIcon,
		action: () => emit('createShortcut'),
	},
])
</script>
