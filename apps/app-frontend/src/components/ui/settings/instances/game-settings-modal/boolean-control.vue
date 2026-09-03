<script setup lang="ts">
import { Button, ButtonGroup } from '@modrinth/ui'

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
	<ButtonGroup v-tooltip="modelValue === undefined ? placeholder : undefined" :label="label">
		<Button
			type="outlined"
			:color="modelValue === true ? 'green' : undefined"
			:aria-pressed="modelValue === true"
			:disabled="disabled"
			@click="emit('update:model-value', true)"
		>
			{{ onLabel }}
		</Button>
		<Button
			:type="modelValue === false ? 'base' : 'outlined'"
			:aria-pressed="modelValue === false"
			:disabled="disabled"
			@click="emit('update:model-value', false)"
		>
			{{ offLabel }}
		</Button>
	</ButtonGroup>
</template>
