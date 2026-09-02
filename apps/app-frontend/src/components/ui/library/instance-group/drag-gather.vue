<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

import DragPreview from '@/components/ui/library/instance-group/drag-preview.vue'
import {
	gatherDuration,
	type InstanceDragGatherItem,
} from '@/components/ui/library/instance-group/use-instance-drag-gather'

const props = defineProps<{
	items: InstanceDragGatherItem[]
	target: {
		x: number
		y: number
	}
}>()

const emit = defineEmits<{
	complete: []
}>()

const gathered = ref(false)
const finishing = ref(false)
let animationFrame: number | undefined
let completionTimer: number | undefined

const itemContainerStyles = computed(() =>
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
			width: `${item.rect.width}px`,
			height: `${item.rect.height}px`,
			transform: gathered.value
				? `translate3d(${-item.rect.width / 2}px, ${-item.rect.height / 2}px, 0)`
				: `translate3d(${originX}px, ${originY}px, 0)`,
			transitionDuration: `${gatherDuration}ms`,
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
				Math.max(gatherDuration - 250, 0),
			)
		})
	})
})

onBeforeUnmount(() => {
	if (animationFrame !== undefined) {
		cancelAnimationFrame(animationFrame)
	}
	if (completionTimer !== undefined) {
		clearTimeout(completionTimer)
	}
})
</script>

<template>
	<Teleport to="body">
		<div
			class="pointer-events-none fixed inset-0 z-[9998] overflow-hidden transition-opacity duration-150 ease-out"
			:class="finishing ? 'opacity-0' : 'opacity-100'"
			aria-hidden="true"
		>
			<div
				v-for="(item, index) in items"
				:key="item.instance.id"
				class="fixed"
				:style="itemContainerStyles[index]"
			>
				<div
					class="origin-center transition-transform ease-[cubic-bezier(0.2,0.8,0.2,1)]"
					:style="itemStyles[index]"
				>
					<DragPreview :instance="item.instance" />
				</div>
			</div>
		</div>
	</Teleport>
</template>
