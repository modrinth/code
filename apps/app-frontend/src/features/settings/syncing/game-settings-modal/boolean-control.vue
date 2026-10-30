<script setup lang="ts">
import { SegmentedControl } from '@modrinth/ui'

const props = defineProps<{
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
	<SegmentedControl
		v-tooltip="modelValue === undefined ? placeholder : undefined"
		:model-value="modelValue"
		:items="[true, false]"
		:format-label="(value) => (value ? props.onLabel : props.offLabel)"
		:neutral-items="[false]"
		:aria-label="label"
		:disabled="disabled"
		@update:model-value="(value) => value !== undefined && emit('update:model-value', value)"
	/>
</template>
