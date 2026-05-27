<script setup lang="ts">
import { computed, useTemplateRef } from 'vue'

const props = withDefaults(
	defineProps<{
		selected?: boolean
		tooltip?: string
		dragActive?: boolean
		dropzone?: boolean
	}>(),
	{
		selected: false,
		tooltip: undefined,
		dragActive: false,
		dropzone: false,
	},
)

const emit = defineEmits<{
	(e: 'click', event: MouseEvent): void
	(e: 'dragenter' | 'dragover' | 'dragleave' | 'drop', event: DragEvent): void
}>()

const root = useTemplateRef<HTMLElement>('root')
const isHighlighted = computed(() => props.selected || props.dragActive)

function handleDragEvent(
	eventName: 'dragenter' | 'dragover' | 'dragleave' | 'drop',
	event: DragEvent,
) {
	if (props.dropzone) {
		event.preventDefault()
		if (event.dataTransfer) {
			event.dataTransfer.dropEffect = 'copy'
		}
	}

	emit(eventName, event)
}

function getRootElement() {
	return root.value
}

defineExpose({ getRootElement })
</script>

<template>
	<div
		ref="root"
		v-tooltip="tooltip ?? undefined"
		class="group relative flex flex-col items-center justify-center overflow-hidden rounded-[20px] border border-dashed transition-[background,border-color,box-shadow] duration-200 focus-within:outline focus-within:outline-2 focus-within:outline-offset-2 focus-within:outline-brand"
		:class="[
			isHighlighted
				? 'border-brand bg-brand-highlight'
				: 'border-surface-5 bg-surface-2 hover:bg-surface-3',
		]"
		@dragenter="handleDragEvent('dragenter', $event)"
		@dragover="handleDragEvent('dragover', $event)"
		@dragleave="handleDragEvent('dragleave', $event)"
		@drop="handleDragEvent('drop', $event)"
	>
		<button
			type="button"
			:aria-label="tooltip ?? undefined"
			class="absolute inset-0 z-0 cursor-pointer border-none bg-transparent p-0"
			@click="(e) => emit('click', e)"
		></button>

		<div
			class="pointer-events-none relative z-10 flex h-full w-full flex-col items-center justify-center gap-4 px-3 text-center"
			:class="dragActive ? 'text-brand' : 'text-contrast'"
		>
			<div v-if="$slots.icon" class="size-8">
				<slot name="icon" />
			</div>
			<div class="flex flex-col items-center gap-0.5 whitespace-nowrap">
				<span class="text-base font-semibold leading-6">
					<slot />
				</span>
				<span v-if="$slots.subtitle" class="text-sm font-medium leading-5 text-primary">
					<slot name="subtitle" />
				</span>
			</div>
		</div>
	</div>
</template>
