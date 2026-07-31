<script setup lang="ts">
import { computed, ref } from 'vue'

import ButtonFrame from './ButtonFrame.vue'
import type {
	ButtonColor,
	ButtonInteraction,
	ButtonNativeType,
	ButtonSize,
	ButtonType,
} from './types'

const props = withDefaults(
	defineProps<{
		label: string
		type?: ButtonType
		color?: ButtonColor
		size?: ButtonSize
		interaction?: ButtonInteraction
		nativeType?: ButtonNativeType
		circular?: boolean
		disabled?: boolean
		loading?: boolean
	}>(),
	{
		type: 'base',
		size: 'md',
		nativeType: 'button',
		circular: true,
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
		:circular="props.circular"
		:type="props.type"
		:color="props.color"
		:size="props.size"
		:interaction="props.interaction"
		:native-type="props.nativeType"
		:disabled="props.disabled || props.loading"
		:aria-label="props.label"
		:aria-busy="props.loading || undefined"
	>
		<slot />
	</ButtonFrame>
</template>
