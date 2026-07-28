<script setup lang="ts">
import { useDraggable } from '@dnd-kit/vue'
import { CheckIcon, DownloadIcon, PlayIcon, SpinnerIcon, StopCircleIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled, injectNotificationManager } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { useMagicKeys } from '@vueuse/core'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import InstanceFileIcon from '@/assets/icons/instance-file.svg'
import { useLibrary } from '@/components/ui/library/use-library'
import { trackEvent } from '@/helpers/analytics'
import { process_listener } from '@/helpers/events'
import { install_existing_instance, install_pack_to_existing_instance } from '@/helpers/install'
import { kill, run } from '@/helpers/instance'
import { get_by_instance_id } from '@/helpers/process'
import type { GameInstance } from '@/helpers/types'
import { showInstanceInFolder } from '@/helpers/utils.js'
import { handleSevereError } from '@/store/error.js'

type ProcessEvent = 'installing' | 'launched' | 'finished'

type ProcessEventPayload = {
	instance_id: string
	event: ProcessEvent
}

const { handleError } = injectNotificationManager()
const {
	displayState,
	selectedLibraryInstanceIds,
	isLibraryInstanceSelectionActive,
	activeDraggedInstanceIds,
} = useLibrary()

const props = defineProps<{
	instance: GameInstance
	instanceGroupName: string
	isSelectionAnchor?: boolean
}>()

const emit = defineEmits<{
	(e: 'toggle-selection', shiftKey: boolean): void
}>()

const instanceCard = ref<HTMLElement | null>(null)
const playing = ref(false)
const loading = ref(false)
const currentEvent = ref<ProcessEvent | null>(null)
const modLoading = computed(
	() =>
		loading.value ||
		currentEvent.value === 'installing' ||
		(currentEvent.value === 'launched' && !playing.value),
)
const installing = computed(() => props.instance.install_stage.includes('installing'))
const installed = computed(() => props.instance.install_stage === 'installed')
const selected = computed(() => selectedLibraryInstanceIds.value.has(props.instance.id))
const keys = useMagicKeys()
const holdingShift = computed(() => keys.shift.value)
const isPartOfActiveDrag = computed(() => activeDraggedInstanceIds.value.has(props.instance.id))
const { isDragging } = useDraggable({
	id: computed(() => `instance:${props.instanceGroupName}:${props.instance.id}`),
	element: instanceCard,
	disabled: computed(() => displayState.value.group !== 'Group'),
	data: computed(() => ({
		instanceId: props.instance.id,
		fromGroup: props.instanceGroupName,
	})),
})

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

const toggleSelection = (event?: MouseEvent) => {
	emit('toggle-selection', event?.shiftKey ?? false)
}

const activateCard = (event: MouseEvent) => {
	if (isLibraryInstanceSelectionActive.value || event.shiftKey) {
		toggleSelection(event)
	} else {
		void seeInstance()
	}
}

const handleCardKeydown = (event: KeyboardEvent) => {
	if (event.target !== event.currentTarget) return

	if (event.key === 'Enter') {
		event.preventDefault()
		if (isLibraryInstanceSelectionActive.value) {
			toggleSelection()
		} else {
			void seeInstance()
		}
	} else if (event.key === ' ' && isLibraryInstanceSelectionActive.value) {
		event.preventDefault()
		toggleSelection()
	}
}

const checkProcess = async () => {
	const runningProcesses = (await get_by_instance_id(props.instance.id).catch(handleError)) ?? []

	playing.value = runningProcesses.length > 0
}

const play = async (event: MouseEvent | null, context: string) => {
	event?.stopPropagation()
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

const stop = async (event: MouseEvent | null, context: string) => {
	event?.stopPropagation()
	playing.value = false

	await kill(props.instance.id).catch(handleError)

	trackEvent('InstanceStop', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
		source: context,
	})
}

const repair = async (event: MouseEvent) => {
	event.stopPropagation()
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

const unlisten = await process_listener((event: ProcessEventPayload) => {
	if (event.instance_id === props.instance.id) {
		currentEvent.value = event.event
		if (event.event === 'finished') {
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
		ref="instanceCard"
		class="group/card relative flex min-h-[76px] w-full cursor-pointer items-center justify-center gap-2 -outline-offset-2 overflow-clip focus-visible:!outline-2 rounded-[20px] border border-solid border-surface-4 bg-surface-3 p-4 text-left transition-all hover:brightness-110 active:scale-[0.98] select-none"
		:class="{
			'border-primary': selected,
			'!scale-100': isDragging,
			'opacity-50': isPartOfActiveDrag,
		}"
		data-library-instance-card
		:data-instance-id="instance.id"
		:data-instance-group="instanceGroupName"
		role="button"
		tabindex="0"
		:aria-label="
			isLibraryInstanceSelectionActive
				? `${selected ? 'Deselect' : 'Select'} ${instance.name}`
				: `Open ${instance.name}`
		"
		:aria-pressed="isLibraryInstanceSelectionActive ? selected : undefined"
		@click="activateCard"
		@keydown="handleCardKeydown"
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
		<button
			type="button"
			class="group/selection absolute right-0 top-0 z-[2] flex size-[50px] h-full cursor-pointer items-start pt-4 justify-center border-0 bg-transparent p-0"
			:aria-label="selected ? 'Deselect instance' : 'Select instance'"
			:aria-pressed="selected"
			@click.stop="toggleSelection"
		>
			<span
				v-tooltip="selected ? 'Deselect instance' : 'Select instance'"
				class="relative flex size-[24px] items-center justify-center rounded-full opacity-0 transition-opacity duration-200 ease-out group-hover/card:opacity-100 group-hover/selection:brightness-125"
				:class="{
					'border-0 !opacity-100': selected,
					'border-2 border-solid border-primary bg-transparent': !selected,
					'[outline:3px_solid_var(--color-purple)] outline-offset-1':
						holdingShift && isSelectionAnchor,
				}"
			>
				<span v-if="selected" class="absolute inset-0 rounded-full bg-primary" />
				<CheckIcon v-if="selected" class="relative size-4 invert [stroke-width:3] top-px" />
			</span>
		</button>
		<div class="relative z-[1] flex min-w-0 flex-1 items-center gap-2 pr-20">
			<div class="relative flex size-10 shrink-0 items-center justify-center">
				<div
					v-if="!playing && !modLoading && !installing"
					class="flex w-10 flex-col items-center gap-px overflow-clip rounded-[14px] px-[3px] py-0.5 text-primary transition-opacity"
					:class="{
						'group-hover/card:scale-75 group-hover/card:opacity-0':
							!instance.quarantined && !isLibraryInstanceSelectionActive,
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
					<ButtonStyled
						v-else-if="!isLibraryInstanceSelectionActive && !installed && !instance.quarantined"
						color="brand"
						circular
					>
						<button
							v-tooltip="'Repair'"
							class="card-shadow origin-bottom scale-75 opacity-0 transition-opacity group-hover/card:scale-100 group-hover/card:opacity-100"
							@click="(e) => repair(e)"
						>
							<DownloadIcon />
						</button>
					</ButtonStyled>
					<ButtonStyled
						v-else-if="!isLibraryInstanceSelectionActive && !instance.quarantined"
						color="brand"
						circular
					>
						<button
							v-tooltip="'Play'"
							class="card-shadow origin-bottom scale-75 opacity-0 transition-opacity group-hover/card:scale-100 group-hover/card:opacity-100"
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
