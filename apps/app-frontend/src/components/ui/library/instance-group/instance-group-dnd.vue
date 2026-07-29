<script setup lang="ts">
import {
	DragDropProvider,
	type DragEndEvent,
	type DragMoveEvent,
	type DragOverEvent,
	DragOverlay,
	type DragStartEvent,
} from '@dnd-kit/vue'
import { computed, onBeforeUnmount, ref, toRef, watch } from 'vue'

import DragGather from '@/components/ui/library/instance-group/drag-gather.vue'
import DragPreview from '@/components/ui/library/instance-group/drag-preview.vue'
import { useInstanceDragGather } from '@/components/ui/library/instance-group/use-instance-drag-gather'
import { useLibrary } from '@/components/ui/library/use-library'
import type { GameInstance } from '@/helpers/types'

type InstanceDragData = {
	instanceId: string
	fromGroup: string
}

type InstanceGroupDndDropData = {
	groupId: string
}

const props = defineProps<{
	instances: GameInstance[]
}>()

const {
	activeInstanceGroupDrag,
	instanceGroupDragPointer,
	instanceGroupDragStatus,
	startInstanceGroupDrag,
	updateInstanceGroupDrag,
	finishInstanceGroupDrag,
	setInstanceGroupDragTarget,
	getInstanceGroupDropState,
	moveDraggedInstancesToGroup,
} = useLibrary()

const draggedInstances = computed(() => {
	const drag = activeInstanceGroupDrag.value
	if (!drag) return []

	const draggedInstanceIds = new Set(drag.instances.map((selection) => selection.instanceId))
	return props.instances.filter((instance) => draggedInstanceIds.has(instance.id))
})
const draggedInstance = computed(() => {
	const drag = activeInstanceGroupDrag.value
	return drag
		? props.instances.find((instance) => instance.id === drag.primaryInstanceId)
		: undefined
})
const {
	items: gatherItems,
	target: gatherTarget,
	isGathering,
	start: startGather,
	updateTarget: updateGatherTarget,
	clear: clearGather,
	finish: finishGather,
} = useInstanceDragGather(toRef(props, 'instances'))

const instanceGroupDragStatusPopover = ref<HTMLElement>()
let statusPopoverFrame: number | undefined

const instanceGroupDragStatusStyle = computed(() => ({
	left: `${Math.min(instanceGroupDragPointer.value.x + 4, window.innerWidth - 220)}px`,
	top: `${Math.min(instanceGroupDragPointer.value.y, window.innerHeight - 48)}px`,
}))

watch(
	activeInstanceGroupDrag,
	(drag) => {
		if (statusPopoverFrame !== undefined) {
			cancelAnimationFrame(statusPopoverFrame)
			statusPopoverFrame = undefined
		}

		const popover = instanceGroupDragStatusPopover.value
		if (!drag) {
			if (popover?.matches(':popover-open')) {
				popover.hidePopover()
			}
			return
		}

		statusPopoverFrame = requestAnimationFrame(() => {
			statusPopoverFrame = undefined
			const currentPopover = instanceGroupDragStatusPopover.value
			if (activeInstanceGroupDrag.value && !currentPopover?.matches(':popover-open')) {
				currentPopover?.showPopover()
			}
		})
	},
	{ flush: 'sync' },
)

onBeforeUnmount(() => {
	if (statusPopoverFrame !== undefined) {
		cancelAnimationFrame(statusPopoverFrame)
	}
})

function isAltKeyPressed(event?: Event) {
	return event instanceof MouseEvent || event instanceof KeyboardEvent ? event.altKey : false
}

function handleDragStart(event: DragStartEvent) {
	const sourceData = event.operation.source?.data as InstanceDragData | undefined
	if (!sourceData) return

	const pointer = event.operation.position.current
	startInstanceGroupDrag(
		sourceData.instanceId,
		sourceData.fromGroup,
		pointer,
		isAltKeyPressed(event.nativeEvent),
	)
	startGather(activeInstanceGroupDrag.value, sourceData, pointer)
}

function handleDragMove(event: DragMoveEvent) {
	const pointer = event.to ?? event.operation.position.current
	updateInstanceGroupDrag(pointer, isAltKeyPressed(event.nativeEvent))
	if (gatherItems.value.length > 0) {
		updateGatherTarget(pointer)
	}
}

function handleDragOver(event: DragOverEvent) {
	const targetData = event.operation.target?.data as InstanceGroupDndDropData | undefined
	setInstanceGroupDragTarget(targetData?.groupId ?? null)
}

function handleDragEnd(event: DragEndEvent) {
	const targetData = event.operation.target?.data as InstanceGroupDndDropData | undefined
	if (!event.canceled && targetData) {
		const dropState = getInstanceGroupDropState(targetData.groupId)
		if (dropState.canDrop) {
			void moveDraggedInstancesToGroup(targetData.groupId, dropState.operation === 'add')
		}
	}

	clearGather()
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
		<slot />
		<Teleport to="body">
			<div class="pointer-events-none fixed inset-0 z-[9999]">
				<DragOverlay :drop-animation="null">
					<div
						v-if="draggedInstance"
						class="w-full transition-all duration-150 ease-out"
						:class="isGathering ? 'scale-[0.975]' : 'scale-100'"
					>
						<DragPreview :instance="draggedInstance" :count="draggedInstances.length" />
					</div>
				</DragOverlay>
			</div>
		</Teleport>
	</DragDropProvider>
	<DragGather
		v-if="gatherItems.length > 0"
		:items="gatherItems"
		:target="gatherTarget"
		@complete="finishGather"
	/>
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
</template>
