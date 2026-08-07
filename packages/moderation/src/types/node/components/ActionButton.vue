<template>
	<Button
		v-tooltip="tooltip"
		:type="color === 'standard' ? 'base' : 'colored'"
		:color="color === 'standard' ? undefined : color"
		:disabled="disabled"
		:aria-label="icon ? label : undefined"
		@click="emit('update:modelValue', !modelValue)"
	>
		<component :is="icon" v-if="icon" />
		<template v-else>{{ label }}</template>
	</Button>
</template>

<script lang="ts" setup>
import { Button } from '@modrinth/ui'
import type { Component } from 'vue'
import { computed } from 'vue'

const props = defineProps<{
	modelValue: boolean
	label?: string
	icon?: Component
	disabled?: boolean
	needsAttention?: boolean
	fixActionable?: boolean
	tooltip?: Record<string, unknown>
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
