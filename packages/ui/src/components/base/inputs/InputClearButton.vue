<template>
	<button
		type="button"
		class="relative flex size-5 shrink-0 touch-manipulation cursor-pointer select-none items-center justify-center border-none bg-transparent p-0 text-secondary opacity-60 transition-colors before:absolute before:-inset-2 hover:text-contrast hover:opacity-100 focus-visible:text-contrast focus-visible:opacity-100 group-focus-within/input:text-contrast group-focus-within/input:opacity-100 disabled:pointer-events-none"
		:aria-label="label"
		:disabled="disabled"
		@click="emit('click', $event)"
	>
		<XIcon class="size-5" aria-hidden="true" />
	</button>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'

withDefaults(
	defineProps<{
		disabled?: boolean
		label?: string
	}>(),
	{
		disabled: false,
		label: 'Clear input',
	},
)

const emit = defineEmits<{
	click: [event: MouseEvent]
}>()
</script>
