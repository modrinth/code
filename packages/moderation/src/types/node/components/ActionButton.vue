<template>
	<ButtonStyled :color="color" :circular="!!icon">
		<button
			v-tooltip="tooltip"
			:disabled="disabled"
			:aria-label="icon ? label : undefined"
			@click="emit('update:modelValue', !modelValue)"
		>
			<component :is="icon" v-if="icon" />
			<template v-else>{{ label }}</template>
		</button>
	</ButtonStyled>
</template>

<script lang="ts" setup>
import { ButtonStyled } from '@modrinth/ui'
import { computed } from 'vue'
import type { Component } from 'vue'

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
