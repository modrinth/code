<script setup lang="ts">
import {
	EyeIcon,
	FolderOpenIcon,
	MoreVerticalIcon,
	PlayIcon,
	SparklesIcon,
	SpinnerIcon,
	StopCircleIcon,
} from '@modrinth/assets'
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
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { capitalizeString } from '@modrinth/utils'
import type { Dayjs } from 'dayjs'
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import { getInstanceIconUrl, kill, run } from '@/helpers/instance'
import { get_by_instance_id } from '@/helpers/process'
import type { GameInstance } from '@/helpers/types'
import { showInstanceInFolder } from '@/helpers/utils'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const router = useRouter()

const emit = defineEmits<{
	(e: 'play' | 'stop'): void
}>()

const props = defineProps<{
	instance: GameInstance
	last_played: Dayjs
	newlyAdded?: boolean
}>()

const messages = defineMessages({
	newInstance: {
		id: 'app.home.jump-back-in.new-instance',
		defaultMessage: 'New instance',
	},
	neverPlayed: {
		id: 'app.home.jump-back-in.never-played',
		defaultMessage: 'Never played',
	},
	lockedTooltip: {
		id: 'app.home.jump-back-in.instance-locked',
		defaultMessage: 'This instance has been locked',
	},
	alreadyOpenTooltip: {
		id: 'app.home.jump-back-in.instance-open',
		defaultMessage: 'Instance is already open',
	},
	moreOptions: {
		id: 'app.home.jump-back-in.more-options',
		defaultMessage: 'More options',
	},
	viewInstance: {
		id: 'app.home.jump-back-in.view-instance',
		defaultMessage: 'View instance',
	},
})

const instanceIcon = computed(() => props.instance.icon_path)

const loader = computed(() => {
	if (props.instance.loader === 'vanilla') {
		return 'Minecraft'
	} else if (props.instance.loader === 'neoforge') {
		return 'NeoForge'
	} else {
		return capitalizeString(props.instance.loader)
	}
})

const loading = ref(false)
const playing = ref(false)

const play = async (event: MouseEvent) => {
	event?.stopPropagation()
	if (props.instance.quarantined) return
	loading.value = true
	const launched = await run(props.instance.id)
		.then(() => true)
		.catch((err) => {
			handleSevereError(err, { instanceId: props.instance.id })
			return false
		})
		.finally(() => {
			trackEvent('InstanceStart', {
				loader: props.instance.loader,
				game_version: props.instance.game_version,
				source: 'InstanceItem',
			})
		})
	if (launched) emit('play')
	loading.value = false
}

const stop = async (event: MouseEvent) => {
	event?.stopPropagation()
	loading.value = true
	await kill(props.instance.id).catch(handleError)
	trackEvent('InstanceStop', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
		source: 'InstanceItem',
	})
	emit('stop')
	loading.value = false
}

useAppEvent('process', async () => {
	await checkProcess()
})

const checkProcess = async () => {
	const runningProcesses = await get_by_instance_id(props.instance.id).catch(handleError)

	playing.value = runningProcesses.length > 0
}

onMounted(() => {
	checkProcess()
})
</script>
<template>
	<SmartClickable class="[--active-scale:0.99]">
		<template #clickable>
			<router-link
				class="no-click-animation"
				:to="`/instance/${encodeURIComponent(instance.id)}`"
			/>
		</template>
		<div
			class="clickable-card grid grid-cols-[auto_minmax(0,3fr)_minmax(0,4fr)_auto] items-center gap-2 border border-surface-4 rounded-[20px] smart-clickable:highlight-on-hover transition-[filter] ease-out [--hover-brightness:1.1] min-h-20 p-3"
			:class="newlyAdded ? 'border-dashed bg-surface-2' : 'bg-bg-raised border-solid'"
		>
			<Avatar
				:src="getInstanceIconUrl(instanceIcon)"
				:tint-by="instance.id"
				no-shadow
				class="!rounded-[14px]"
				size="48px"
			/>
			<div class="flex flex-col col-span-2 justify-center gap-1 h-full">
				<div class="flex items-center gap-1.5">
					<div class="text-contrast truncate text-base font-semibold">
						{{ instance.name }}
					</div>
					<TagItem
						v-if="newlyAdded"
						class="!border-green !bg-bg-green !px-2 !font-medium !text-green"
					>
						<SparklesIcon aria-hidden="true" />
						{{ formatMessage(messages.newInstance) }}
					</TagItem>
				</div>
				<div class="flex items-center gap-1.5 text-sm text-secondary">
					<span class="flex items-center gap-1 truncate text-secondary">
						{{ loader }}
						{{ instance.game_version }}
					</span>
					<BulletDivider class="shrink-0" />
					<div
						v-tooltip="!newlyAdded ? formatDateTime(last_played.toDate()) : null"
						class="w-fit shrink-0"
						:class="{
							'cursor-help smart-clickable:allow-pointer-events': !newlyAdded,
						}"
					>
						<template v-if="newlyAdded">
							{{ formatMessage(messages.neverPlayed) }}
						</template>
						<template v-else-if="last_played">
							{{ formatRelativeTime(last_played.toISOString?.()) }}
						</template>
						<template v-else>{{ formatMessage(messages.neverPlayed) }}</template>
					</div>
				</div>
			</div>
			<div data-no-card-click class="flex gap-1 justify-end smart-clickable:allow-pointer-events">
				<Button v-if="playing && !loading" type="colored" color="red" @click="stop">
					<StopCircleIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.stopButton) }}
				</Button>
				<Button
					v-else
					v-tooltip="
						instance.quarantined
							? formatMessage(messages.lockedTooltip)
							: playing
								? formatMessage(messages.alreadyOpenTooltip)
								: null
					"
					:disabled="instance.quarantined || playing || loading"
					type="colored"
					color="green"
					@click="play"
				>
					<SpinnerIcon v-if="loading" class="animate-spin" />
					<PlayIcon v-else aria-hidden="true" />
					{{ formatMessage(commonMessages.playButton) }}
				</Button>
				<TeleportOverflowMenu
					type="quiet"
					:label="formatMessage(messages.moreOptions)"
					:options="[
						{
							id: 'open-instance',
							label: formatMessage(messages.viewInstance),
							shown: !!instance.id,
							action: () => router.push(encodeURI(`/instance/${instance.id}`)),
						},
						{
							id: 'open-folder',
							label: formatMessage(commonMessages.openFolderButton),
							action: () => showInstanceInFolder(instance.id),
						},
					]"
				>
					<MoreVerticalIcon aria-hidden="true" />
					<template #open-instance>
						<EyeIcon aria-hidden="true" />
						{{ formatMessage(messages.viewInstance) }}
					</template>
					<template #open-folder>
						<FolderOpenIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.openFolderButton) }}
					</template>
				</TeleportOverflowMenu>
			</div>
		</div>
	</SmartClickable>
</template>
<style scoped>
.clickable-card:has([data-no-card-click]:hover) {
	--hover-brightness: 1;
}
</style>
