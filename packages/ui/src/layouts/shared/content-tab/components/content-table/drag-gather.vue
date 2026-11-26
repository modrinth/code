<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

import DragPreview from './drag-preview.vue'
import { ROW_GATHER_DURATION, type RowDragGatherItem } from './use-row-drag-gather'

const props = defineProps<{
	items: RowDragGatherItem[]
	target: { x: number; y: number }
	teleportTo: HTMLElement | string
}>()

const emit = defineEmits<{
	complete: []
}>()

const gathered = ref(false)
const finishing = ref(false)
let animationFrame: number | undefined
let completionTimer: number | undefined

const containerStyles = computed(() =>
	props.items.map((_, index) => ({
		left: `${props.target.x}px`,
		top: `${props.target.y}px`,
		zIndex: `${index}`,
	})),
)

const itemStyles = computed(() =>
	props.items.map((item, index) => {
		const originX = item.rect.left - props.target.x
		const originY = item.rect.top - props.target.y
		return {
			transform: gathered.value
				? 'translate3d(-50%, -50%, 0)'
				: `translate3d(${originX}px, ${originY}px, 0)`,
			transitionDuration: `${ROW_GATHER_DURATION}ms`,
			transitionDelay: `${Math.min(index, 8) * 10}ms`,
		}
	}),
)

onMounted(() => {
	animationFrame = requestAnimationFrame(() => {
		animationFrame = requestAnimationFrame(() => {
			gathered.value = true
			completionTimer = window.setTimeout(
				() => {
					finishing.value = true
					emit('complete')
				},
				Math.max(ROW_GATHER_DURATION - 250, 0),
			)
		})
	})
})

onBeforeUnmount(() => {
	if (animationFrame !== undefined) cancelAnimationFrame(animationFrame)
	if (completionTimer !== undefined) clearTimeout(completionTimer)
})
</script>

<template>
	<Teleport :to="teleportTo">
		<div
			class="pointer-events-none fixed inset-0 z-[9998] overflow-hidden transition-opacity duration-150 ease-out"
			:class="finishing ? 'opacity-0' : 'opacity-100'"
			aria-hidden="true"
		>
			<div
				v-for="(item, index) in items"
				:key="item.row.id"
				class="fixed"
				:style="containerStyles[index]"
			>
				<div
					class="transition-transform ease-[cubic-bezier(0.2,0.8,0.2,1)]"
					:style="itemStyles[index]"
				>
					<DragPreview :row="item.row" />
				</div>
			</div>
		</div>
	</Teleport>
</template>
