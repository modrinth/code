<script setup lang="ts">
import { PointerActivationConstraints } from '@dnd-kit/dom'
import { KeyboardSensor, PointerSensor, useDraggable, useDroppable } from '@dnd-kit/vue'
import { ref } from 'vue'

const props = defineProps<{
	rowId: string
	sectionId: string
	disabled: boolean
}>()

const element = ref<HTMLElement | null>(null)

const sensors = [
	PointerSensor.configure({
		activationConstraints: [new PointerActivationConstraints.Distance({ value: 5 })],
	}),
	KeyboardSensor,
]

useDraggable({
	id: () => `content-row:${props.rowId}`,
	element,
	disabled: () => props.disabled,
	sensors,
	data: () => ({ rowId: props.rowId }),
})

useDroppable({
	id: () => `content-row-drop:${props.rowId}`,
	element,
	data: () => ({ sectionId: props.sectionId }),
})
</script>

<template>
	<div ref="element">
		<slot />
	</div>
</template>
