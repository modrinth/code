<script setup lang="ts">
import {
	ClipboardCopyIcon,
	EditIcon,
	ExternalIcon,
	EyeIcon,
	FolderOpenIcon,
	IssuesIcon,
	MoreVerticalIcon,
	NoSignalIcon,
	PlayIcon,
	SignalIcon,
	SkullIcon,
	SpinnerIcon,
	StopCircleIcon,
	TrashIcon,
	UpdatedIcon,
	UserIcon,
	XIcon,
} from '@modrinth/assets'
import type { MessageDescriptor } from '@modrinth/ui'
import {
	Avatar,
	BulletDivider,
	Button,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	SmartClickable,
	TagItem,
	TeleportOverflowMenu,
	useFormatDateTime,
	useFormatNumber,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { getPingLevel } from '@modrinth/utils'
import dayjs from 'dayjs'
import { Tooltip } from 'floating-vue'
import type { Component } from 'vue'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import { getInstanceIconUrl } from '@/helpers/instance'
import { copyToClipboard, createInstanceShortcut } from '@/helpers/utils'
import type {
	ProtocolVersion,
	ServerStatus,
	ServerWorld,
	SingleplayerWorld,
	World,
} from '@/helpers/worlds.ts'
import { getWorldIdentifier, set_world_display_status } from '@/helpers/worlds.ts'

import { LockIcon } from '../../../../../../packages/assets/generated-icons'

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatNumber = useFormatNumber()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const router = useRouter()
const { addNotification } = injectNotificationManager()

const emit = defineEmits<{
	(e: 'play' | 'play-instance' | 'update' | 'stop' | 'refresh' | 'edit' | 'delete'): void
	(e: 'open-folder', world: SingleplayerWorld): void
}>()

const props = withDefaults(
	defineProps<{
		world: World
		playingInstance?: boolean
		playingWorld?: boolean
		startingInstance?: boolean
		supportsServerQuickPlay?: boolean
		supportsWorldQuickPlay?: boolean
		quarantined?: boolean
		currentProtocol?: ProtocolVersion | null
		highlighted?: boolean

		// Server only
		refreshing?: boolean
		serverStatus?: ServerStatus
		renderedMotd?: string

		// Singleplayer only
		gameMode?: {
			icon: Component
			message: MessageDescriptor
		}

		managed?: boolean

		// Instance
		instanceId?: string
		instanceName?: string
		instanceIcon?: string
		shortcutInstanceId?: string
	}>(),
	{
		playingInstance: false,
		playingWorld: false,
		startingInstance: false,
		supportsServerQuickPlay: true,
		supportsWorldQuickPlay: false,
		quarantined: false,
		currentProtocol: null,

		refreshing: false,
		serverStatus: undefined,
		renderedMotd: undefined,

		gameMode: undefined,
		managed: false,

		instanceId: undefined,
		instanceName: undefined,
		instanceIcon: undefined,
		shortcutInstanceId: undefined,
	},
)

const playingOtherWorld = computed(() => props.playingInstance && !props.playingWorld)
const hasPlayersTooltip = computed(
	() => !!props.serverStatus?.players?.sample && props.serverStatus.players.sample.length > 0,
)
const serverIncompatible = computed(
	() =>
		!!props.serverStatus &&
		!!props.serverStatus.version?.protocol &&
		!!props.currentProtocol &&
		(props.serverStatus.version.protocol !== props.currentProtocol.version ||
			props.serverStatus.version.legacy !== props.currentProtocol.legacy),
)

const locked = computed(() => props.world.type === 'singleplayer' && props.world.locked)
const managed = computed(() => props.managed)
const shortcutInstanceId = computed(() => props.shortcutInstanceId ?? props.instanceId)

async function createShortcut() {
	if (!shortcutInstanceId.value || props.quarantined) return

	try {
		const shortcutPath = await createInstanceShortcut(
			props.world.name,
			shortcutInstanceId.value,
			props.world.type === 'server'
				? { server: (props.world as ServerWorld).address }
				: { singleplayerWorld: (props.world as SingleplayerWorld).path },
		)
		if (!shortcutPath) return

		addNotification({
			type: 'success',
			title: formatMessage(messages.shortcutCreated),
		})
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.shortcutCreationFailed),
			text: error instanceof Error ? error.message : '',
		})
	}
}

const messages = defineMessages({
	shortcutCreated: {
		id: 'instance.worlds.shortcut-created',
		defaultMessage: 'Shortcut created',
	},
	shortcutCreationFailed: {
		id: 'instance.worlds.shortcut-creation-failed',
		defaultMessage: 'Failed to create shortcut',
	},
	moreOptions: { id: 'instance.worlds.more-options', defaultMessage: 'More options' },
	instanceLocked: {
		id: 'instance.worlds.instance-locked',
		defaultMessage: 'This instance has been locked',
	},
	hardcore: {
		id: 'instance.worlds.hardcore',
		defaultMessage: 'Hardcore mode',
	},
	cantConnect: {
		id: 'instance.worlds.cant_connect',
		defaultMessage: "Can't connect to server",
	},
	aMinecraftServer: {
		id: 'instance.worlds.a_minecraft_server',
		defaultMessage: 'A Minecraft Server',
	},
	noServerQuickPlay: {
		id: 'instance.worlds.no_server_quick_play',
		defaultMessage: 'You can only jump straight into servers on Minecraft Alpha 1.0.5+',
	},
	noSingleplayerQuickPlay: {
		id: 'instance.worlds.no_singleplayer_quick_play',
		defaultMessage: 'You can only jump straight into singleplayer worlds on Minecraft 1.20+',
	},
	gameAlreadyOpen: {
		id: 'instance.worlds.game_already_open',
		defaultMessage: 'Instance is already open',
	},
	noContact: {
		id: 'instance.worlds.no_contact',
		defaultMessage: "Server couldn't be contacted",
	},
	incompatibleServer: {
		id: 'instance.worlds.incompatible_server',
		defaultMessage: 'Server is incompatible',
	},
	copyAddress: {
		id: 'instance.worlds.copy_address',
		defaultMessage: 'Copy address',
	},
	viewInstance: {
		id: 'instance.worlds.view_instance',
		defaultMessage: 'View instance',
	},
	playInstance: {
		id: 'instance.worlds.play_instance',
		defaultMessage: 'Play instance',
	},
	worldInUse: {
		id: 'instance.worlds.world_in_use',
		defaultMessage: 'World is in use',
	},
	dontShowOnHome: {
		id: 'instance.worlds.dont_show_on_home',
		defaultMessage: `Don't show on Home`,
	},
	createShortcut: {
		id: 'instance.worlds.create_shortcut',
		defaultMessage: 'Create shortcut',
	},
	linkedServer: {
		id: 'instance.worlds.linked_server',
		defaultMessage: 'Managed by server project',
	},
	incompatibleVersion: {
		id: 'app.world.world-item.incompatible-version',
		defaultMessage: 'Incompatible version {version}',
	},
	playersOnline: {
		id: 'app.world.world-item.players-online',
		defaultMessage: '{count} online',
	},
	offline: {
		id: 'app.world.world-item.offline',
		defaultMessage: 'Offline',
	},
	notPlayedYet: {
		id: 'app.world.world-item.not-played-yet',
		defaultMessage: 'Not played yet',
	},
})
</script>
<template>
	<SmartClickable class="[--active-scale:0.985]">
		<template v-if="instanceId" #clickable>
			<router-link
				class="no-click-animation"
				:to="`/instance/${encodeURIComponent(instanceId)}/worlds?highlight=${encodeURIComponent(getWorldIdentifier(world))}`"
			/>
		</template>
		<div
			class="clickable-card grid grid-cols-[auto_minmax(0,3fr)_minmax(0,4fr)_auto] items-center gap-2 p-3 bg-bg-raised border border-solid border-surface-4 smart-clickable:highlight-on-hover rounded-[20px] transition-[filter] ease-out [--hover-brightness:1.25] min-h-20"
			:class="{
				'world-item-highlighted': highlighted,
			}"
		>
			<Avatar
				:src="
					world.type === 'server' && serverStatus
						? (serverStatus.favicon ?? world.icon)
						: world.icon
				"
				size="48px"
				no-shadow
				class="!rounded-[14px]"
			/>
			<div class="flex flex-col justify-center gap-0.5 h-full">
				<div class="flex items-center gap-1.5">
					<div class="text-base text-contrast font-semibold truncate">
						{{ world.name }}
					</div>
					<TagItem
						v-if="managed"
						v-tooltip="formatMessage(messages.linkedServer)"
						class="border !border-solid border-blue bg-highlight-blue text-xs"
						:style="`--_color: var(--color-blue)`"
					>
						<LockIcon aria-hidden="true" class="h-5 w-5" />
					</TagItem>
					<div
						v-if="world.type === 'singleplayer'"
						class="text-sm text-secondary flex items-center gap-1 font-semibold flex-nowrap whitespace-nowrap"
					>
						<UserIcon
							aria-hidden="true"
							class="h-4 w-4 text-secondary shrink-0"
							stroke-width="3px"
						/>
						{{ formatMessage(commonMessages.singleplayerLabel) }}
					</div>
					<div
						v-else-if="world.type === 'server'"
						class="text-sm text-secondary flex items-center gap-1 font-semibold flex-nowrap whitespace-nowrap"
					>
						<template v-if="refreshing">
							<SpinnerIcon aria-hidden="true" class="animate-spin shrink-0" />
							{{ formatMessage(commonMessages.loadingLabel) }}
						</template>
						<template v-else-if="serverStatus">
							<template v-if="serverIncompatible">
								<IssuesIcon class="shrink-0 text-orange" aria-hidden="true" />
								<span class="text-orange">
									{{
										formatMessage(messages.incompatibleVersion, {
											version: serverStatus.version?.name,
										})
									}}
								</span>
							</template>
							<template v-else>
								<SignalIcon
									v-tooltip="`${serverStatus.ping}ms`"
									aria-hidden="true"
									:style="`--_signal-${getPingLevel(serverStatus.ping ?? 0)}: var(--color-green)`"
									stroke-width="3px"
									class="shrink-0 smart-clickable:allow-pointer-events"
								/>
								<Tooltip :disabled="!hasPlayersTooltip">
									<span
										class="smart-clickable:allow-pointer-events"
										:class="{ 'cursor-help': hasPlayersTooltip }"
									>
										{{
											formatMessage(messages.playersOnline, {
												count: formatNumber(serverStatus.players?.online ?? 0),
											})
										}}
									</span>
									<template #popper>
										<div class="flex flex-col gap-1">
											<span v-for="player in serverStatus.players?.sample" :key="player.id">
												{{ player.name }}
											</span>
										</div>
									</template>
								</Tooltip>
							</template>
						</template>
						<template v-else>
							<NoSignalIcon aria-hidden="true" stroke-width="3px" class="shrink-0" />
							{{ formatMessage(messages.offline) }}
						</template>
					</div>
				</div>
				<div class="flex items-center gap-1.5 text-sm text-secondary">
					<template v-if="instanceId">
						<router-link
							data-no-card-click
							class="flex items-center gap-1 truncate hover:underline text-secondary smart-clickable:allow-pointer-events"
							:to="`/instance/${instanceId}`"
						>
							<Avatar
								:src="getInstanceIconUrl(instanceIcon)"
								size="16px"
								:tint-by="instanceId"
								class="shrink-0"
								no-shadow
							/>
							<span class="truncate">{{ instanceName }}</span>
						</router-link>
						<BulletDivider class="shrink-0" />
					</template>
					<div
						v-tooltip="world.last_played ? formatDateTime(world.last_played) : null"
						class="w-fit shrink-0"
						:class="{
							'cursor-help smart-clickable:allow-pointer-events': world.last_played,
						}"
					>
						<template v-if="world.last_played">
							{{ formatRelativeTime(dayjs(world.last_played).toISOString()) }}
						</template>
						<template v-else> {{ formatMessage(messages.notPlayedYet) }} </template>
					</div>
				</div>
			</div>
			<div
				class="font-semibold flex items-center gap-1 justify-center text-center"
				:class="world.type === 'singleplayer' && world.hardcore ? `text-red` : 'text-secondary'"
			>
				<template v-if="world.type === 'server'">
					<template v-if="refreshing">
						<SpinnerIcon aria-hidden="true" class="animate-spin" />
						{{ formatMessage(commonMessages.loadingLabel) }}
					</template>
					<div
						v-else-if="renderedMotd"
						class="motd-renderer font-normal font-minecraft line-clamp-2 text-secondary leading-5"
						v-html="renderedMotd"
					/>
					<div v-else-if="!serverStatus" class="font-normal font-minecraft text-red leading-5">
						{{ formatMessage(messages.cantConnect) }}
					</div>
					<div v-else class="font-normal font-minecraft text-secondary leading-5">
						{{ formatMessage(messages.aMinecraftServer) }}
					</div>
				</template>
				<template v-else-if="world.type === 'singleplayer' && gameMode">
					<template v-if="world.hardcore">
						<SkullIcon aria-hidden="true" class="h-4 w-4 shrink-0" />
						{{ formatMessage(messages.hardcore) }}
					</template>
					<template v-else>
						<component :is="gameMode.icon" aria-hidden="true" class="h-4 w-4 shrink-0" />
						{{ formatMessage(gameMode.message) }}
					</template>
				</template>
			</div>
			<div class="flex gap-1 justify-end smart-clickable:allow-pointer-events">
				<Button
					v-if="(playingWorld || (locked && playingInstance)) && !startingInstance"
					type="colored"
					color="red"
					@click="emit('stop')"
				>
					<StopCircleIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.stopButton) }}
				</Button>
				<Button
					v-else
					v-tooltip="
						quarantined
							? formatMessage(messages.instanceLocked)
							: world.type === 'server'
								? !supportsServerQuickPlay
									? formatMessage(messages.noServerQuickPlay)
									: playingOtherWorld
										? formatMessage(messages.gameAlreadyOpen)
										: !serverStatus
											? formatMessage(messages.noContact)
											: serverIncompatible
												? formatMessage(messages.incompatibleServer)
												: null
								: !supportsWorldQuickPlay
									? formatMessage(messages.noSingleplayerQuickPlay)
									: playingOtherWorld || locked
										? formatMessage(messages.gameAlreadyOpen)
										: null
					"
					:disabled="
						quarantined ||
						playingOtherWorld ||
						startingInstance ||
						(world.type == 'server' && !supportsServerQuickPlay) ||
						(world.type == 'singleplayer' && !supportsWorldQuickPlay)
					"
					type="colored"
					color="brand"
					@click="emit('play')"
				>
					<SpinnerIcon v-if="startingInstance && playingWorld" class="animate-spin" />
					<PlayIcon v-else aria-hidden="true" />
					{{ formatMessage(commonMessages.playButton) }}
				</Button>
				<TeleportOverflowMenu
					type="quiet"
					:label="formatMessage(messages.moreOptions)"
					:options="[
						{
							id: 'play-instance',
							label: formatMessage(messages.playInstance),
							shown: !!instanceId,
							disabled: playingInstance || quarantined,
							action: () => emit('play-instance'),
						},
						{
							id: 'open-instance',
							label: formatMessage(messages.viewInstance),
							shown: !!instanceId,
							action: () => router.push(`/instance/${encodeURIComponent(instanceId)}`),
						},
						{
							id: 'refresh',
							label: formatMessage(commonMessages.refreshButton),
							shown: world.type === 'server',
							action: () => emit('refresh'),
						},
						{
							id: 'copy-address',
							label: formatMessage(messages.copyAddress),
							shown: world.type === 'server',
							action: () => copyToClipboard((world as ServerWorld).address),
						},
						{
							id: 'edit',
							label: formatMessage(commonMessages.editButton),
							action: () => emit('edit'),
							shown: !instanceId,
							disabled: locked || managed,
							tooltip: locked
								? formatMessage(messages.worldInUse)
								: managed
									? formatMessage(messages.linkedServer)
									: undefined,
						},
						{
							id: 'open-folder',
							label: formatMessage(commonMessages.openFolderButton),
							shown: world.type === 'singleplayer',
							action: () => (world.type === 'singleplayer' ? emit('open-folder', world) : {}),
						},
						{
							type: 'divider',
							shown: !!instanceId,
						},
						{
							id: 'dont-show-on-home',
							label: formatMessage(messages.dontShowOnHome),
							shown: !!instanceId,
							action: () => {
								set_world_display_status(
									instanceId,
									world.type,
									getWorldIdentifier(world),
									'hidden',
								).then(() => {
									emit('update')
								})
							},
						},
						{
							id: 'create-shortcut',
							label: formatMessage(messages.createShortcut),
							shown: !!shortcutInstanceId && !quarantined,
							action: () => createShortcut(),
						},
						{
							type: 'divider',
							shown: !instanceId,
						},
						{
							id: 'delete',
							label: formatMessage(
								world.type === 'server' ? commonMessages.removeButton : commonMessages.deleteLabel,
							),
							tone: 'red',
							action: () => emit('delete'),
							shown: !instanceId,
							disabled: locked || managed,
							tooltip: locked
								? formatMessage(messages.worldInUse)
								: managed
									? formatMessage(messages.linkedServer)
									: undefined,
						},
					]"
				>
					<MoreVerticalIcon aria-hidden="true" />
					<template #play-instance>
						<PlayIcon aria-hidden="true" />
						{{ formatMessage(messages.playInstance) }}
					</template>
					<template #open-instance>
						<EyeIcon aria-hidden="true" />
						{{ formatMessage(messages.viewInstance) }}
					</template>
					<template #edit>
						<EditIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.editButton) }}
					</template>
					<template #open-folder>
						<FolderOpenIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.openFolderButton) }}
					</template>
					<template #copy-address>
						<ClipboardCopyIcon aria-hidden="true" />
						{{ formatMessage(messages.copyAddress) }}
					</template>
					<template #refresh>
						<UpdatedIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.refreshButton) }}
					</template>
					<template #create-shortcut>
						<ExternalIcon aria-hidden="true" />
						{{ formatMessage(messages.createShortcut) }}
					</template>
					<template #dont-show-on-home>
						<XIcon aria-hidden="true" />
						{{ formatMessage(messages.dontShowOnHome) }}
					</template>
					<template #delete>
						<TrashIcon aria-hidden="true" />
						{{
							formatMessage(
								world.type === 'server' ? commonMessages.removeButton : commonMessages.deleteLabel,
							)
						}}
					</template>
				</TeleportOverflowMenu>
			</div>
		</div>
	</SmartClickable>
</template>
<style scoped lang="scss">
.clickable-card:has([data-no-card-click]:hover) {
	--hover-brightness: 1;
}

.world-item-highlighted {
	position: relative;
	animation: fade-highlight 4s ease-out;
	filter: brightness(1);

	&::before {
		@apply rounded-xl inset-0 absolute;

		animation: fade-opacity 4s ease-out;

		content: '';
		box-shadow: 0 0 8px 2px var(--color-brand);
		border: 1.5px solid var(--color-brand);
		opacity: 0;
	}
}

@keyframes fade-highlight {
	0% {
		filter: brightness(1.25);
	}
	75% {
		filter: brightness(1.25);
	}
	100% {
		filter: brightness(1);
	}
}

@keyframes fade-opacity {
	0% {
		opacity: 0.5;
	}
	75% {
		opacity: 0.5;
	}
	100% {
		opacity: 0;
	}
}

.light-mode .motd-renderer {
	filter: brightness(0.75);
}
</style>
