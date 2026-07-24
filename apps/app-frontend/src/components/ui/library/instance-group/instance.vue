<script setup>
import { DownloadIcon, PlayIcon, SpinnerIcon, StopCircleIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled, injectNotificationManager } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import InstanceFileIcon from '@/assets/icons/instance-file.svg?component'
import { trackEvent } from '@/helpers/analytics'
import { process_listener } from '@/helpers/events'
import { install_existing_instance, install_pack_to_existing_instance } from '@/helpers/install'
import { kill, run } from '@/helpers/instance'
import { get_by_instance_id } from '@/helpers/process'
import { showInstanceInFolder } from '@/helpers/utils.js'
import { handleSevereError } from '@/store/error.js'

const { handleError } = injectNotificationManager()

const props = defineProps({
	instance: {
		type: Object,
		default() {
			return {}
		},
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
const instanceType = computed(() => {
	if (
		props.instance.link?.type === 'server_project' ||
		props.instance.link?.type === 'server_project_modpack'
	) {
		return 'SRV'
	}

	return props.instance.link?.type === 'modrinth_modpack' ? 'MPK' : 'CST'
})

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

const unlisten = await process_listener((e) => {
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
onUnmounted(() => unlisten())
</script>

<template>
	<div
		class="group relative flex min-h-[76px] w-full cursor-pointer items-center justify-center gap-2 overflow-clip rounded-[20px] border border-solid border-surface-4 bg-surface-3 p-4 text-left shadow-[0_1px_1px_0_rgba(0,0,0,0.12)] transition-all hover:brightness-110 active:scale-[0.98]"
		@click="seeInstance"
		@mouseenter="checkProcess"
	>
		<Avatar
			class="pointer-events-none !border-none !bg-transparent !rounded-[26px] !rounded-br-[42px] !absolute -top-[40px] right-[18px] opacity-50 [mask-image:linear-gradient(135deg,transparent_16%,black_100%)]"
			size="100px"
			:src="instance.icon_path ? convertFileSrc(instance.icon_path) : null"
			:tint-by="instance.id"
			alt=""
			no-shadow
		/>
		<div class="relative z-[1] flex min-w-0 flex-1 items-center gap-2 pr-20">
			<div class="relative flex size-10 shrink-0 items-center justify-center">
				<div
					v-if="!playing && !modLoading && !installing"
					class="flex w-10 flex-col items-center gap-px overflow-clip rounded-[14px] px-[3px] py-0.5 text-primary transition-all"
					:class="{
						'group-hover:scale-75 group-hover:opacity-0 group-focus-within:scale-75 group-focus-within:opacity-0':
							!instance.quarantined,
					}"
				>
					<InstanceFileIcon class="h-[21px] w-[31px] shrink-0 text-primary [&_path]:fill-current" />
					<span class="h-3.5 text-sm font-extrabold leading-[13px]">{{ instanceType }}</span>
				</div>
				<div class="absolute inset-0 flex items-center justify-center">
					<ButtonStyled v-if="playing" color="red" circular>
						<button
							v-tooltip="'Stop'"
							class="card-shadow"
							@click="(e) => stop(e, 'InstanceCard')"
							@mouseenter="checkProcess"
						>
							<StopCircleIcon />
						</button>
					</ButtonStyled>
					<SpinnerIcon
						v-else-if="modLoading || installing"
						v-tooltip="modLoading ? 'Instance is loading...' : 'Installing...'"
						class="size-8 animate-spin"
						tabindex="-1"
					/>
					<ButtonStyled v-else-if="!installed && !instance.quarantined" color="brand" circular>
						<button
							v-tooltip="'Repair'"
							class="card-shadow origin-bottom scale-75 opacity-0 transition-opacity group-hover:scale-100 group-hover:opacity-100 group-focus-within:scale-100 group-focus-within:opacity-100"
							@click="(e) => repair(e)"
						>
							<DownloadIcon />
						</button>
					</ButtonStyled>
					<ButtonStyled v-else-if="!instance.quarantined" color="brand" circular>
						<button
							v-tooltip="'Play'"
							class="card-shadow origin-bottom scale-75 opacity-0 transition-opacity group-hover:scale-100 group-hover:opacity-100 group-focus-within:scale-100 group-focus-within:opacity-100"
							@click="(e) => play(e, 'InstanceCard')"
							@mouseenter="checkProcess"
						>
							<PlayIcon class="translate-x-px" />
						</button>
					</ButtonStyled>
				</div>
			</div>
			<div class="flex min-w-0 flex-1 flex-col justify-center gap-1">
				<p class="m-0 truncate text-base font-semibold leading-5 text-contrast">
					{{ instance.name }}
				</p>
				<p class="m-0 truncate text-sm font-medium capitalize leading-[18px] text-primary">
					{{ instance.loader }} {{ instance.game_version }}
				</p>
			</div>
		</div>
	</div>
</template>
