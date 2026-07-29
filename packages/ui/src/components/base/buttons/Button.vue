<script setup lang="ts">
import { computed, ref } from 'vue'

import ButtonFrame from './ButtonFrame.vue'
import type { ButtonColor, ButtonNativeType, ButtonSize, ButtonType } from './types'

const props = withDefaults(
	defineProps<{
		type?: ButtonType
		color?: ButtonColor
		size?: ButtonSize
		nativeType?: ButtonNativeType
		disabled?: boolean
		loading?: boolean
	}>(),
	{
		type: 'base',
		size: 'default',
		nativeType: 'button',
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
		:type="props.type"
		:color="props.color"
		:size="props.size"
		:native-type="props.nativeType"
		:disabled="props.disabled || props.loading"
		:aria-busy="props.loading || undefined"
	>
		<slot />
	</ButtonFrame>
</template>
