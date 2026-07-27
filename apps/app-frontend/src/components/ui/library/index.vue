<script setup lang="ts">
import {
	DragDropProvider,
	type DragEndEvent,
	type DragMoveEvent,
	type DragOverEvent,
	DragOverlay,
	type DragStartEvent,
} from '@dnd-kit/vue'
import {
	ClipboardCopyIcon,
	EyeIcon,
	FolderOpenIcon,
	MinusIcon,
	PlayIcon,
	PlusIcon,
	StopCircleIcon,
	TrashIcon,
} from '@modrinth/assets'
import { computed, onBeforeUnmount, ref, toRef, watch } from 'vue'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import InstanceDragPreview from '@/components/ui/library/instance-group/instance-drag-preview.vue'
import InstanceGroup from '@/components/ui/library/instance-group/index.vue'
import LibrarySelectionActionBar from '@/components/ui/library/LibrarySelectionActionBar.vue'
import LibraryToolbar from '@/components/ui/library/library-toolbar/index.vue'
import { provideLibrary } from '@/components/ui/library/use-library'
import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'
import type { GameInstance } from '@/helpers/types'

const props = defineProps<{
	instances: GameInstance[]
}>()

const {
	instanceGroups,
	activeInstanceGroupDrag,
	instanceGroupDragPointer,
	instanceGroupDragStatus,
	instanceOptions,
	confirmDeleteModal,
	startInstanceGroupDrag,
	updateInstanceGroupDrag,
	finishInstanceGroupDrag,
	setInstanceGroupDragTarget,
	getInstanceGroupDropState,
	moveDraggedInstanceToGroup,
	deleteInstance,
	handleInstanceOption,
} = provideLibrary(toRef(props, 'instances'))

type InstanceDragData = {
	instanceId: string
	fromGroup: string
}

type InstanceGroupDropData = {
	groupName: string
}

const draggedInstance = computed(() =>
	activeInstanceGroupDrag.value
		? props.instances.find((instance) => instance.id === activeInstanceGroupDrag.value?.instanceId)
		: undefined,
)

const instanceGroupDragStatusPopover = ref<HTMLElement>()
let showInstanceGroupDragStatusFrame: number | undefined

const instanceGroupDragStatusStyle = computed(() => ({
	left: `${Math.min(instanceGroupDragPointer.value.x + 4, window.innerWidth - 220)}px`,
	top: `${Math.min(instanceGroupDragPointer.value.y, window.innerHeight - 48)}px`,
}))

watch(
	activeInstanceGroupDrag,
	(drag) => {
		if (showInstanceGroupDragStatusFrame !== undefined) {
			cancelAnimationFrame(showInstanceGroupDragStatusFrame)
			showInstanceGroupDragStatusFrame = undefined
		}

		const popover = instanceGroupDragStatusPopover.value
		if (!drag) {
			if (popover?.matches(':popover-open')) {
				popover.hidePopover()
			}
			return
		}

		showInstanceGroupDragStatusFrame = requestAnimationFrame(() => {
			showInstanceGroupDragStatusFrame = undefined
			const currentPopover = instanceGroupDragStatusPopover.value
			if (activeInstanceGroupDrag.value && !currentPopover?.matches(':popover-open')) {
				currentPopover?.showPopover()
			}
		})
	},
	{ flush: 'sync' },
)

onBeforeUnmount(() => {
	if (showInstanceGroupDragStatusFrame !== undefined) {
		cancelAnimationFrame(showInstanceGroupDragStatusFrame)
	}
})

function isAltKeyPressed(event?: Event) {
	return event instanceof MouseEvent || event instanceof KeyboardEvent ? event.altKey : false
}

function handleDragStart(event: DragStartEvent) {
	const sourceData = event.operation.source?.data as InstanceDragData | undefined
	if (!sourceData) return

	startInstanceGroupDrag(
		sourceData.instanceId,
		sourceData.fromGroup,
		event.operation.position.current,
		isAltKeyPressed(event.nativeEvent),
	)
}

function handleDragMove(event: DragMoveEvent) {
	updateInstanceGroupDrag(
		event.to ?? event.operation.position.current,
		isAltKeyPressed(event.nativeEvent),
	)
}

function handleDragOver(event: DragOverEvent) {
	const targetData = event.operation.target?.data as InstanceGroupDropData | undefined
	setInstanceGroupDragTarget(targetData?.groupName ?? null)
}

function handleDragEnd(event: DragEndEvent) {
	const targetData = event.operation.target?.data as InstanceGroupDropData | undefined
	if (!event.canceled && targetData) {
		const dropState = getInstanceGroupDropState(targetData.groupName)
		if (dropState.canDrop) {
			void moveDraggedInstanceToGroup(targetData.groupName, dropState.operation === 'add')
		}
	}

	finishInstanceGroupDrag()
}
</script>

<template>
	<DragDropProvider
		@drag-start="handleDragStart"
		@drag-move="handleDragMove"
		@drag-over="handleDragOver"
		@drag-end="handleDragEnd"
	>
		<section class="flex flex-col gap-3">
			<h2 class="m-0 text-2xl font-semibold text-contrast">Library</h2>
			<LibraryToolbar />
			<div class="flex flex-col">
				<InstanceGroup
					v-for="instanceGroup in instanceGroups"
					:key="instanceGroup.id"
					:instance-group="instanceGroup"
				/>
			</div>
		</section>
		<DragOverlay :drop-animation="null">
			<InstanceDragPreview v-if="draggedInstance" :instance="draggedInstance" />
		</DragOverlay>
	</DragDropProvider>
	<Teleport to="body">
		<div
			ref="instanceGroupDragStatusPopover"
			popover="manual"
			class="pointer-events-none fixed inset-auto z-[9999] m-0 rounded-xl border border-solid border-surface-5 bg-surface-4 px-2 py-1.5 text-sm font-semibold text-contrast empty:hidden"
			:style="instanceGroupDragStatusStyle"
		>
			{{ instanceGroupDragStatus }}
		</div>
	</Teleport>
	<LibrarySelectionActionBar />
	<ConfirmDeleteInstanceModal ref="confirmDeleteModal" @delete="deleteInstance" />
	<ContextMenu ref="instanceOptions" @option-clicked="handleInstanceOption">
		<template #play> <PlayIcon /> Play </template>
		<template #stop> <StopCircleIcon /> Stop </template>
		<template #add_content> <PlusIcon /> Add content </template>
		<template #edit> <EyeIcon /> View instance </template>
		<template #duplicate> <ClipboardCopyIcon /> Duplicate instance</template>
		<template #delete> <TrashIcon /> Delete </template>
		<template #open> <FolderOpenIcon /> Open folder </template>
		<template #copy> <ClipboardCopyIcon /> Copy path </template>
		<template #remove_from_group> <MinusIcon /> Remove from group </template>
	</ContextMenu>
</template>
