<template>
	<IconButton
		v-if="icon"
		:type="color === 'standard' ? 'base' : 'colored'"
		:color="color === 'standard' ? undefined : color"
		:disabled="disabled"
		:label="label"
		@click="emit('update:modelValue', !modelValue)"
	>
		<component :is="icon" aria-hidden="true" />
	</IconButton>
	<Button
		v-else
		:type="color === 'standard' ? 'base' : 'colored'"
		:color="color === 'standard' ? undefined : color"
		:disabled="disabled"
		@click="emit('update:modelValue', !modelValue)"
	>
		{{ label }}
	</Button>
</template>

<script lang="ts" setup>
import { Button, IconButton } from '@modrinth/ui'
import type { Component } from 'vue'
import { computed } from 'vue'

const props = defineProps<{
	modelValue: boolean
	label: string
	icon?: Component
	disabled?: boolean
	needsAttention?: boolean
	fixActionable?: boolean
}>()

const emit = defineEmits<{
	'update:modelValue': [boolean]
}>()

const color = computed(() => {
	if (!props.modelValue) return 'standard'
	if (props.needsAttention) return 'orange'
	return props.fixActionable ? 'blue' : 'brand'
})
</script>
