<template>
	<Button
		v-tooltip="tooltip"
		:type="color === 'standard' ? 'base' : 'colored'"
		:color="color === 'standard' ? undefined : color"
		:disabled="disabled"
		:aria-label="icon ? label : undefined"
		@click="emit('update:modelValue', !modelValue)"
		:size="props.size"
		:interaction="props.interaction"
		:nativeType="props.nativeType"
		:loadin="props.loading"

	>
		<component :is="icon" v-if="icon" />
		<template v-else>{{ label }}</template>
	</Button>
</template>

<script lang="ts" setup>
import { Button } from '@modrinth/ui'
import type { Component } from 'vue'
import { computed } from 'vue'
import type {ButtonProps} from "@modrinth/ui/src/components/base/buttons/types.ts";

const props = defineProps<{
	modelValue: boolean
	label?: string
	icon?: Component
	needsAttention?: boolean
	fixActionable?: boolean
	tooltip?: Record<string, unknown>
} & Omit<ButtonProps, "type" | "color">>()

const emit = defineEmits<{
	'update:modelValue': [boolean]
}>()

const color = computed(() => {
	if (!props.modelValue) return 'standard'
	if (props.needsAttention) return 'orange'
	return props.fixActionable ? 'blue' : 'brand'
})
</script>
