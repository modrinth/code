<script setup lang="ts">
import { computed, ref } from 'vue'

import ButtonFrame from './ButtonFrame.vue'
import type { ButtonNativeType, ButtonSize, ButtonTone, ButtonVariant } from './types'

const props = withDefaults(
	defineProps<{
		label: string
		variant?: ButtonVariant
		tone?: ButtonTone
		size?: ButtonSize
		type?: ButtonNativeType
		disabled?: boolean
		loading?: boolean
	}>(),
	{
		variant: 'base',
		size: 'default',
		type: 'button',
		disabled: false,
		loading: false,
	},
)

const frame = ref<InstanceType<typeof ButtonFrame> | null>(null)
const element = computed(() => frame.value?.element ?? null)

defineExpose({ element })
</script>

<template>
	<ButtonFrame
		ref="frame"
		as="button"
		icon-only
		:variant="props.variant"
		:tone="props.tone"
		:size="props.size"
		:type="props.type"
		:disabled="props.disabled || props.loading"
		:aria-label="props.label"
		:aria-busy="props.loading || undefined"
	>
		<slot />
	</ButtonFrame>
</template>
