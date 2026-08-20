<script setup lang="ts">
import { KeyboardSensor, PointerSensor, useDraggable } from '@dnd-kit/vue'
import { CheckIcon, DownloadIcon, PlayIcon, SpinnerIcon, StopCircleIcon } from '@modrinth/assets'
import { defineMessages, IconButton, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useEventListener, useMagicKeys } from '@vueuse/core'
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import InstanceCardView from '@/components/ui/library/instance-group/instance-card-view.vue'
import { getLibraryInstanceSelectionKey, useLibrary } from '@/components/ui/library/use-library'
import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import { install_existing_instance, install_pack_to_existing_instance } from '@/helpers/install'
import { kill, run } from '@/helpers/instance'
import { get_by_instance_id } from '@/helpers/process'
import type { GameInstance } from '@/helpers/types'
import { showInstanceInFolder } from '@/helpers/utils.js'

type ProcessEvent = 'installing' | 'launched' | 'finished'

const LOADING_INDICATOR_DELAY_MS = 100

const instanceCardSensors = [
	PointerSensor.configure({
		preventActivation: () => false,
	}),
	KeyboardSensor,
]

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	selectInstance: {
		id: 'app.library.instance.select-with-name',
		defaultMessage: 'Select {name}',
	},
	deselectInstance: {
		id: 'app.library.instance.deselect-with-name',
		defaultMessage: 'Deselect {name}',
	},
	openInstance: {
		id: 'app.library.instance.open-with-name',
		defaultMessage: 'Open {name}',
	},
	loading: { id: 'app.library.instance.loading', defaultMessage: 'Instance is loading...' },
	installing: { id: 'app.library.instance.installing', defaultMessage: 'Installing...' },
	stop: { id: 'app.library.instance.stop', defaultMessage: 'Stop' },
	repair: { id: 'app.library.instance.repair', defaultMessage: 'Repair' },
	play: { id: 'app.library.instance.play', defaultMessage: 'Play' },
	select: { id: 'app.library.instance.select', defaultMessage: 'Select instance' },
	deselect: { id: 'app.library.instance.deselect', defaultMessage: 'Deselect instance' },
})
const {
	displayState,
	selectedLibraryInstances,
	isLibraryInstanceSelectionActive,
	activeDraggedInstanceKeys,
} = useLibrary()

const props = defineProps<{
	instance: GameInstance
	instanceGroupId: string
	isSelectionAnchor?: boolean
}>()

const emit = defineEmits<{
	(e: 'toggle-selection', shiftKey: boolean): void
}>()

const instanceCardElement = ref<InstanceType<typeof InstanceCardView> | null>(null)
const playing = ref(false)
const loading = ref(false)
const currentEvent = ref<ProcessEvent | null>(null)
const isPrimaryPointerDown = ref(false)
const modLoading = computed(
	() =>
		loading.value ||
		currentEvent.value === 'installing' ||
		(currentEvent.value === 'launched' && !playing.value),
)
const installing = computed(() => props.instance.install_stage.includes('installing'))
const installed = computed(() => props.instance.install_stage === 'installed')
const loadingIndicatorRequested = computed(
	() => !playing.value && (modLoading.value || installing.value),
)
const loadingIndicatorVisible = ref(false)

watch(
	loadingIndicatorRequested,
	(requested, _, onCleanup) => {
		loadingIndicatorVisible.value = false
		if (!requested) return

		const timeout = setTimeout(() => {
			loadingIndicatorVisible.value = true
		}, LOADING_INDICATOR_DELAY_MS)
		onCleanup(() => clearTimeout(timeout))
	},
	{ immediate: true },
)
const selectionKey = computed(() =>
	getLibraryInstanceSelectionKey({
		instanceId: props.instance.id,
		groupId: props.instanceGroupId,
	}),
)
const selected = computed(() => selectedLibraryInstances.value.has(selectionKey.value))
const keys = useMagicKeys()
const holdingShift = computed(() => keys.shift.value)
const isPartOfActiveDrag = computed(() => activeDraggedInstanceKeys.value.has(selectionKey.value))
const { isDragging } = useDraggable({
	id: computed(() => `instance:${props.instanceGroupId}:${props.instance.id}`),
	element: instanceCardElement,
	disabled: computed(() => displayState.value.group !== 'Group'),
	sensors: instanceCardSensors,
	data: computed(() => ({
		instanceId: props.instance.id,
		fromGroup: props.instanceGroupId,
	})),
})

const router = useRouter()

const seeInstance = async () => {
	await router.push(`/instance/${encodeURIComponent(props.instance.id)}`)
}

const toggleSelection = (event?: MouseEvent) => {
	emit('toggle-selection', event?.shiftKey ?? false)
}

const handlePointerDown = (event: PointerEvent) => {
	isPrimaryPointerDown.value =
		event.button === 0 &&
		!(event.target instanceof Element && event.target.closest('.selection-button'))
}

const resetPrimaryPointer = () => {
	isPrimaryPointerDown.value = false
}

useEventListener(window, 'pointerup', resetPrimaryPointer)
useEventListener(window, 'pointercancel', resetPrimaryPointer)
useEventListener(window, 'blur', resetPrimaryPointer)

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

useAppEvent('process', (event) => {
	if (event.instance_id === props.instance.id) {
		currentEvent.value = event.event
		playing.value = event.event === 'launched'
	}
})

onMounted(() => {
	void checkProcess()
})
</script>

<template>
	<InstanceCardView
		ref="instanceCardElement"
		class="group/card cursor-pointer -outline-offset-2 focus-visible:!outline-2 hover:brightness-110"
		:class="{
			'!scale-100': isDragging,
			'opacity-50': isPartOfActiveDrag,
			'scale-[0.95]': isPrimaryPointerDown && !isLibraryInstanceSelectionActive,
		}"
		:instance="instance"
		:selected="selected"
		data-library-instance-card
		:data-instance-id="instance.id"
		:data-instance-group="instanceGroupId"
		role="button"
		tabindex="0"
		:aria-label="
			isLibraryInstanceSelectionActive
				? formatMessage(selected ? messages.deselectInstance : messages.selectInstance, {
						name: instance.name,
					})
				: formatMessage(messages.openInstance, { name: instance.name })
		"
		:aria-pressed="isLibraryInstanceSelectionActive ? selected : undefined"
		@click="activateCard"
		@keydown="handleCardKeydown"
		@mouseenter="checkProcess"
		@pointerdown="handlePointerDown"
	>
		<template #loading="{ compact }">
			<div
				v-if="loadingIndicatorVisible"
				class="pointer-events-none absolute inset-0 z-[1] flex items-center justify-center"
			>
				<div class="absolute inset-0 bg-surface-1 opacity-30" />
				<SpinnerIcon
					v-tooltip="formatMessage(modLoading ? messages.loading : messages.installing)"
					class="relative animate-spin text-contrast"
					:class="compact ? 'size-5' : 'size-[30%]'"
					tabindex="-1"
				/>
			</div>
		</template>
		<template #leading="{ compact }">
			<div
				class="relative flex shrink-0 items-center justify-center"
				:class="compact ? 'size-10' : 'size-12'"
			>
				<div class="absolute inset-0 flex items-center justify-center">
					<IconButton
						v-if="playing"
						v-tooltip="formatMessage(messages.stop)"
						:label="formatMessage(messages.stop)"
						type="colored"
						color="red"
						:size="compact ? 'md' : 'lg'"
						@click="(e) => stop(e, 'InstanceCard')"
						@mouseenter="checkProcess"
					>
						<StopCircleIcon />
					</IconButton>
					<IconButton
						v-else-if="
							!modLoading &&
							!installing &&
							!isLibraryInstanceSelectionActive &&
							!installed &&
							!instance.quarantined
						"
						v-tooltip="formatMessage(messages.repair)"
						:label="formatMessage(messages.repair)"
						type="colored"
						color="brand"
						:size="compact ? 'md' : 'lg'"
						class="origin-bottom scale-75 opacity-0 transition-opacity group-hover/card:scale-100 group-hover/card:opacity-100"
						@click="(e) => repair(e)"
					>
						<DownloadIcon />
					</IconButton>
					<IconButton
						v-else-if="
							!modLoading &&
							!installing &&
							!isLibraryInstanceSelectionActive &&
							!instance.quarantined
						"
						v-tooltip="formatMessage(messages.play)"
						:label="formatMessage(messages.play)"
						type="colored"
						color="brand"
						:size="compact ? 'md' : 'lg'"
						class="origin-bottom scale-75 opacity-0 transition-opacity group-hover/card:scale-100 group-hover/card:opacity-100"
						@click="(e) => play(e, 'InstanceCard')"
						@mouseenter="checkProcess"
					>
						<PlayIcon class="translate-x-px" />
					</IconButton>
				</div>
			</div>
		</template>
		<template #overlay="{ compact }">
			<button
				type="button"
				class="selection-button group/selection absolute z-[2] flex size-[50px] cursor-pointer items-start pt-4 justify-center border-0 bg-transparent p-0"
				:class="compact ? '-right-1 -top-1' : 'right-2 top-1.5'"
				:aria-label="formatMessage(selected ? messages.deselect : messages.select)"
				:aria-pressed="selected"
				@click.stop="toggleSelection"
			>
				<span
					v-tooltip="formatMessage(selected ? messages.deselect : messages.select)"
					class="relative flex items-center justify-center rounded-full opacity-0 transition-opacity duration-200 ease-out group-hover/card:opacity-100 group-hover/selection:brightness-125"
					:class="{
						'size-[20px]': compact,
						'size-[24px]': !compact,
						'border-0 !opacity-100': selected,
						'border-2 border-solid border-primary bg-transparent': !selected,
						'[outline:3px_solid_var(--color-purple)] outline-offset-1':
							holdingShift && isSelectionAnchor,
					}"
				>
					<span v-if="selected" class="absolute inset-0 rounded-full bg-contrast" />
					<CheckIcon v-if="selected" class="relative size-4 invert [stroke-width:3]" />
				</span>
			</button>
		</template>
	</InstanceCardView>
</template>
