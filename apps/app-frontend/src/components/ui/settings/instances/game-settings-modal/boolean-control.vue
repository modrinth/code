<script setup lang="ts">
import { Button } from '@modrinth/ui'

defineProps<{
	modelValue?: boolean
	disabled?: boolean
	label: string
	onLabel: string
	offLabel: string
	placeholder?: string
}>()

const emit = defineEmits<{
	'update:model-value': [value: boolean]
}>()
</script>

<template>
	<div
		v-tooltip="modelValue === undefined ? placeholder : undefined"
		role="group"
		:aria-label="label"
		class="inline-flex h-8 items-center gap-0.5 rounded-xl border border-solid border-surface-5 p-px shadow-[0_1px_3px_rgba(0,0,0,0.15)]"
		:class="{ 'opacity-50': disabled }"
	>
		<Button
			type="quiet"
			size="xs"
			:interaction="modelValue === true ? 'none' : 'surface'"
			class="boolean-control-on !rounded-[10px] !px-3 !font-medium disabled:!opacity-100"
			:class="modelValue === true ? '!text-green' : '!text-contrast'"
			:aria-pressed="modelValue === true"
			:disabled="disabled"
			@click="emit('update:model-value', true)"
		>
			{{ onLabel }}
		</Button>
		<Button
			type="quiet"
			size="xs"
			:interaction="modelValue === false ? 'none' : 'surface'"
			class="boolean-control-off !rounded-[10px] !px-3 !font-medium !text-contrast disabled:!opacity-100"
			:aria-pressed="modelValue === false"
			:disabled="disabled"
			@click="emit('update:model-value', false)"
		>
			{{ offLabel }}
		</Button>
	</div>
</template>

<style scoped>
.boolean-control-on[aria-pressed='true'] {
	background-color: color-mix(in srgb, var(--color-green) 30%, var(--surface-3));
	box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-green) 60%, transparent);
}

.boolean-control-off[aria-pressed='true'] {
	background-color: var(--surface-4);
	box-shadow: inset 0 0 0 1px var(--surface-5);
}
</style>
