<template>
	<PageHeader :title="instance.name">
		<template #leading>
			<PageHeaderObjectAvatarLeading
				:src="iconSrc"
				:alt="instance.name"
				size="64px"
				:tint-by="instance.id"
			/>
		</template>

		<template #metadata>
			<InstanceHeaderServerMetadata
				v-if="isServerInstance"
				:loading-server-ping="loadingServerPing"
				:players-online="playersOnline"
				:status-online="statusOnline"
				:recent-plays="recentPlays"
				:ping="ping"
				:minecraft-server="minecraftServer"
				:linked-project-v3="linkedProjectV3"
				:instance-id="instance.id"
			/>
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
				<PageHeaderMetadataItem v-if="showInstancePlayTime" :icon="TimerIcon" tooltip="Total playtime">
					{{ playtimeLabel }}
				</PageHeaderMetadataItem>
			</PageHeaderMetadata>
		</template>

		<template #actions>
			<PageHeaderActions>
				<ButtonStyled v-if="isInstalling" color="brand" size="large">
					<button type="button" disabled>Installing...</button>
				</ButtonStyled>
				<ButtonStyled v-else-if="instance.install_stage !== 'installed'" color="brand" size="large">
					<button type="button" @click="emit('repair')">
						<DownloadIcon />
						Repair
					</button>
				</ButtonStyled>
				<ButtonStyled v-else-if="playing" color="red" size="large">
					<button type="button" :disabled="stopping" @click="emit('stop')">
						<StopCircleIcon />
						{{ stopping ? 'Stopping...' : 'Stop' }}
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
						Play
					</button>
				</ButtonStyled>
				<ButtonStyled v-else color="brand" size="large">
					<button type="button" disabled>Starting...</button>
				</ButtonStyled>

				<ButtonStyled circular size="large">
					<button
						v-tooltip="'Instance settings'"
						type="button"
						aria-label="Instance settings"
						@click="emit('settings')"
					>
						<SettingsIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled circular size="large" type="transparent">
					<TeleportOverflowMenu
						:options="moreActions"
						tooltip="More actions"
						aria-label="More actions"
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
} from '@modrinth/assets'
import {
	ButtonStyled,
	JoinedButtons,
	LoaderIcon as ServerLoaderIcon,
	PageHeader,
	PageHeaderActions,
	PageHeaderMetadata,
	PageHeaderMetadataItem,
	PageHeaderObjectAvatarLeading,
	TeleportOverflowMenu,
	type JoinedButtonAction,
	type ServerLoader,
	type TeleportOverflowMenuItem,
} from '@modrinth/ui'
import { computed } from 'vue'

import type { GameInstance } from '@/helpers/types'

import InstanceHeaderServerMetadata from './instance-page-header-server-metadata.vue'

const props = withDefaults(
	defineProps<{
		instance: GameInstance
		iconSrc?: string | null
		isServerInstance?: boolean
		loaderDisplayName?: ServerLoader | null
		loaderLabel?: string
		showInstancePlayTime?: boolean
		playtimeLabel?: string
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
	}>(),
	{
		iconSrc: null,
		isServerInstance: false,
		loaderDisplayName: null,
		loaderLabel: '',
		showInstancePlayTime: false,
		playtimeLabel: '',
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

const isInstalling = computed(() => installingStages.includes(props.instance.install_stage))
const serverPlayActions = computed<JoinedButtonAction[]>(() => [
	{
		id: 'join_server',
		label: 'Play',
		icon: PlayIcon,
		action: () => emit('playServer'),
	},
	{
		id: 'launch_instance',
		label: 'Launch instance',
		icon: PlayIcon,
		action: () => emit('play'),
	},
])
const moreActions = computed<TeleportOverflowMenuItem[]>(() => [
	{
		id: 'open-folder',
		label: 'Open folder',
		icon: FolderOpenIcon,
		action: () => emit('openFolder'),
	},
	{
		id: 'export-mrpack',
		label: 'Export modpack',
		icon: PackageIcon,
		action: () => emit('export'),
	},
	{
		id: 'create-shortcut',
		label: 'Create shortcut',
		icon: ExternalIcon,
		action: () => emit('createShortcut'),
	},
])
</script>
