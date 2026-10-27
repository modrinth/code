<script setup lang="ts">
import { PointerActivationConstraints } from '@dnd-kit/dom'
import { KeyboardSensor, PointerSensor, useDraggable } from '@dnd-kit/vue'
import { DownloadIcon, PlayIcon, SpinnerIcon, StopCircleIcon } from '@modrinth/assets'
import { defineMessages, IconButton, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useEventListener } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import InstanceCardView from '@/components/ui/library/instance-group/instance-card-view.vue'
import InstanceOwnerBadge from '@/components/ui/library/instance-group/instance-owner-badge.vue'
import InstanceSelectionButton from '@/components/ui/library/instance-group/instance-selection-button.vue'
import { getLibraryInstanceSelectionKey, useLibrary } from '@/components/ui/library/use-library'
import { useInstanceActions } from '@/features/instances/use-instance-actions'
import { libraryInstanceMessages } from '@/features/library/messages'
import { install_existing_instance, install_pack_to_existing_instance } from '@/helpers/install'
import type { GameInstance } from '@/helpers/types'

const LOADING_INDICATOR_DELAY_MS = 100

const instanceCardSensors = [
	PointerSensor.configure({
		activationConstraints: [new PointerActivationConstraints.Distance({ value: 5 })],
	}),
	KeyboardSensor,
]

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	loading: { id: 'app.library.instance.loading', defaultMessage: 'Instance is loading...' },
	stop: { id: 'app.library.instance.stop', defaultMessage: 'Stop' },
	repair: { id: 'app.library.instance.repair', defaultMessage: 'Repair' },
	play: { id: 'app.library.instance.play', defaultMessage: 'Play' },
})
const {
	displayState,
	selectedLibraryInstances,
	isLibraryInstanceSelectionActive,
	activeDraggedInstanceKeys,
	canSelectLibraryInstance,
	getSharedInstanceOwner,
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
const isPrimaryPointerDown = ref(false)
const {
	playing,
	modLoading,
	installing,
	checkProcess,
	play,
	stop,
	seeInstance,
	openFolder,
	addContent,
} = useInstanceActions(() => props.instance)
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
const selectable = computed(() => selected.value || canSelectLibraryInstance(props.instance.id))
const owner = computed(() => getSharedInstanceOwner(props.instance))
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
		if (selectable.value) toggleSelection(event)
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

const repair = async (event: MouseEvent) => {
	event.stopPropagation()
	if (props.instance.quarantined || installing.value || modLoading.value) return

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

defineExpose({
	play,
	stop,
	seeInstance,
	openFolder,
	addContent,
	instance: props.instance,
	get playing() {
		return playing.value
	},
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
			'!cursor-default opacity-50': !selectable,
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
				? formatMessage(
						selected
							? libraryInstanceMessages.deselectWithName
							: libraryInstanceMessages.selectWithName,
						{
							name: instance.name,
						},
					)
				: formatMessage(libraryInstanceMessages.openWithName, { name: instance.name })
		"
		:aria-pressed="isLibraryInstanceSelectionActive ? selected : undefined"
		@click="activateCard"
		@keydown="handleCardKeydown"
		@mouseenter="checkProcess"
		@pointerdown="handlePointerDown"
	>
		<template #icon-overlay="{ compact }">
			<InstanceOwnerBadge v-if="owner" :owner="owner" :compact="compact" />
		</template>
		<template #loading="{ compact }">
			<div
				v-if="loadingIndicatorVisible"
				class="pointer-events-none absolute inset-0 z-[1] flex items-center justify-center"
			>
				<div class="absolute inset-0 bg-surface-1 opacity-30" />
				<SpinnerIcon
					v-tooltip="
						formatMessage(modLoading ? messages.loading : libraryInstanceMessages.installing)
					"
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
			<InstanceSelectionButton
				v-if="selectable"
				:selected="selected"
				:compact="compact"
				:is-selection-anchor="isSelectionAnchor"
				@toggle="toggleSelection"
			/>
		</template>
	</InstanceCardView>
</template>
