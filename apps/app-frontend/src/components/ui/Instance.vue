<script setup>
import {
	DownloadIcon,
	GameIcon,
	PlayIcon,
	SpinnerIcon,
	StopCircleIcon,
	TimerIcon,
} from '@modrinth/assets'
import {
	Avatar,
	defineMessages,
	IconButton,
	injectNotificationManager,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import dayjs from 'dayjs'
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import { install_existing_instance, install_pack_to_existing_instance } from '@/helpers/install'
import { getInstanceIconUrl, kill, run } from '@/helpers/instance'
import { get_by_instance_id } from '@/helpers/process'
import { showInstanceInFolder } from '@/helpers/utils.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const messages = defineMessages({
	instanceIcon: { id: 'app.instance.card.icon-alt', defaultMessage: 'Instance icon' },
	stop: { id: 'app.instance.card.stop', defaultMessage: 'Stop' },
	loading: { id: 'app.instance.card.loading', defaultMessage: 'Instance is loading...' },
	installing: { id: 'app.instance.card.installing', defaultMessage: 'Installing...' },
	play: { id: 'app.instance.card.play', defaultMessage: 'Play' },
	repair: { id: 'app.instance.card.repair', defaultMessage: 'Repair' },
	played: { id: 'app.instance.card.played', defaultMessage: 'Played {relativeTime}' },
	neverPlayed: { id: 'app.instance.card.never-played', defaultMessage: 'Never played' },
})

const props = defineProps({
	instance: {
		type: Object,
		default() {
			return {}
		},
	},
	compact: {
		type: Boolean,
		default: false,
	},
	first: {
		type: Boolean,
		default: false,
	},
})

const playing = ref(false)
const loading = ref(false)
const modLoading = computed(
	() =>
		loading.value ||
		currentEvent.value === 'installing' ||
		(currentEvent.value === 'launched' && !playing.value),
)
const installing = computed(() => props.instance.install_stage.includes('installing'))
const installed = computed(() => props.instance.install_stage === 'installed')

const router = useRouter()

const seeInstance = async () => {
	await router.push(`/instance/${encodeURIComponent(props.instance.id)}`)
}

const checkProcess = async () => {
	const runningProcesses = await get_by_instance_id(props.instance.id).catch(handleError)

	playing.value = runningProcesses.length > 0
}

const play = async (e, context) => {
	e?.stopPropagation()
	if (props.instance.quarantined) return
	loading.value = true
	await run(props.instance.id)
		.catch((err) => handleSevereError(err, { instanceId: props.instance.id }))
		.finally(() => {
			trackEvent('InstanceStart', {
				loader: props.instance.loader,
				game_version: props.instance.game_version,
				source: context,
			})
		})
	loading.value = false
}

const stop = async (e, context) => {
	e?.stopPropagation()
	playing.value = false

	await kill(props.instance.id).catch(handleError)

	trackEvent('InstanceStop', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
		source: context,
	})
}

const repair = async (e) => {
	e?.stopPropagation()
	if (props.instance.quarantined) return

	if (
		props.instance.install_stage !== 'pack_installed' &&
		(props.instance.link?.type === 'modrinth_modpack' ||
			props.instance.link?.type === 'server_project_modpack')
	) {
		await install_pack_to_existing_instance(props.instance.id, {
			type: 'fromVersionId',
			project_id: props.instance.link.project_id ?? props.instance.link.server_project_id ?? '',
			version_id: props.instance.link.version_id ?? props.instance.link.content_version_id ?? '',
			title: props.instance.name,
		}).catch(handleError)
	} else {
		await install_existing_instance(props.instance.id, false).catch(handleError)
	}
}

const openFolder = async () => {
	await showInstanceInFolder(props.instance.id)
}

const addContent = async () => {
	if (props.instance.quarantined) return
	await router.push({
		path: `/browse/${props.instance.loader === 'vanilla' ? 'datapack' : 'mod'}`,
		query: { i: props.instance.id },
	})
}

defineExpose({
	play,
	stop,
	seeInstance,
	openFolder,
	addContent,
	instance: props.instance,
})

const currentEvent = ref(null)

useAppEvent('process', (e) => {
	if (e.instance_id === props.instance.id) {
		currentEvent.value = e.event
		if (e.event === 'finished') {
			playing.value = false
		}
	}
})

onMounted(() => {
	checkProcess()
})
</script>

<template>
	<template v-if="compact">
		<div
			class="card-shadow grid grid-cols-[auto_1fr_auto] bg-bg-raised rounded-xl p-3 pl-4 gap-2 cursor-pointer hover:brightness-90 transition-all"
			@click="seeInstance"
			@mouseenter="checkProcess"
		>
			<Avatar
				size="48px"
				:src="getInstanceIconUrl(instance.icon_path)"
				:tint-by="instance.id"
				:alt="formatMessage(messages.instanceIcon)"
			/>
			<div class="h-full flex items-center font-bold text-contrast leading-normal">
				<span class="line-clamp-2">{{ instance.name }}</span>
			</div>
			<div class="flex items-center">
				<IconButton
					v-if="playing"
					v-tooltip="formatMessage(messages.stop)"
					type="colored"
					color="red"
					:label="formatMessage(messages.stop)"
					@mouseenter="checkProcess"
					@click="(e) => stop(e, 'InstanceCard')"
				>
					<StopCircleIcon />
				</IconButton>
				<IconButton
					v-else-if="modLoading"
					v-tooltip="formatMessage(messages.loading)"
					:label="formatMessage(messages.loading)"
					disabled
				>
					<SpinnerIcon class="animate-spin" />
				</IconButton>
				<IconButton
					v-else-if="!instance.quarantined"
					v-tooltip="formatMessage(messages.play)"
					:type="first ? 'colored' : 'base'"
					:color="first ? 'brand' : undefined"
					:label="formatMessage(messages.play)"
					@click="(e) => play(e, 'InstanceCard')"
					@mouseenter="checkProcess"
				>
					<!-- Translate for optical centering -->
					<PlayIcon class="translate-x-[1px]" />
				</IconButton>
			</div>
			<div class="flex items-center col-span-3 gap-1 text-secondary font-semibold">
				<TimerIcon />
				<span class="text-sm">
					<template v-if="instance.last_played">
						{{
							formatMessage(messages.played, {
								relativeTime: formatRelativeTime(dayjs(instance.last_played).toISOString()),
							})
						}}
					</template>
					<template v-else>{{ formatMessage(messages.neverPlayed) }}</template>
				</span>
			</div>
		</div>
	</template>
	<div v-else>
		<div
			class="button-base bg-bg-raised p-4 rounded-xl flex gap-3 group"
			@click="seeInstance"
			@mouseenter="checkProcess"
		>
			<div class="relative flex items-center justify-center">
				<Avatar
					size="48px"
					:src="getInstanceIconUrl(instance.icon_path)"
					:tint-by="instance.id"
					:alt="formatMessage(messages.instanceIcon)"
					:class="`transition-all ${modLoading || installing ? `brightness-[0.25] scale-[0.85]` : `group-hover:brightness-75`}`"
				/>
				<div class="absolute inset-0 flex items-center justify-center">
					<IconButton
						v-if="playing"
						v-tooltip="formatMessage(messages.stop)"
						type="colored"
						color="red"
						size="xl"
						:label="formatMessage(messages.stop)"
						:class="{ 'scale-100 opacity-100': playing }"
						class="transition-all scale-75 origin-bottom opacity-0 card-shadow"
						@click="(e) => stop(e, 'InstanceCard')"
						@mouseenter="checkProcess"
					>
						<StopCircleIcon />
					</IconButton>
					<SpinnerIcon
						v-else-if="modLoading || installing"
						v-tooltip="formatMessage(modLoading ? messages.loading : messages.installing)"
						class="animate-spin w-8 h-8"
						tabindex="-1"
					/>
					<IconButton
						v-else-if="!installed && !instance.quarantined"
						v-tooltip="formatMessage(messages.repair)"
						type="colored"
						color="brand"
						size="xl"
						:label="formatMessage(messages.repair)"
						class="transition-all scale-75 group-hover:scale-100 group-focus-within:scale-100 origin-bottom opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 card-shadow"
						@click="(e) => repair(e)"
					>
						<DownloadIcon />
					</IconButton>
					<IconButton
						v-else-if="!instance.quarantined"
						v-tooltip="formatMessage(messages.play)"
						type="colored"
						color="brand"
						size="xl"
						:label="formatMessage(messages.play)"
						class="transition-all scale-75 group-hover:scale-100 group-focus-within:scale-100 origin-bottom opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 card-shadow"
						@click="(e) => play(e, 'InstanceCard')"
						@mouseenter="checkProcess"
					>
						<PlayIcon class="translate-x-[2px]" />
					</IconButton>
				</div>
			</div>
			<div class="flex flex-col gap-1">
				<p class="m-0 text-md font-bold text-contrast leading-tight line-clamp-1">
					{{ instance.name }}
				</p>
				<div class="flex items-center col-span-3 gap-1 text-secondary font-semibold mt-auto">
					<GameIcon class="shrink-0" />
					<span class="text-sm capitalize">
						{{ instance.loader }} {{ instance.game_version }}
					</span>
				</div>
			</div>
		</div>
	</div>
</template>
