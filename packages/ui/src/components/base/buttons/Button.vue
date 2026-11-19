<script setup lang="ts">
import { computed, ref } from 'vue'

import ButtonFrame from './ButtonFrame.vue'
import type { ButtonProps } from './types'

const props = withDefaults(defineProps<ButtonProps>(), {
	type: 'base',
	size: 'md',
	nativeType: 'button',
	disabled: false,
	loading: false,
	animateWidth: false,
})

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
		:interaction="props.interaction"
		:native-type="props.nativeType"
		:animate-width="props.animateWidth"
		:disabled="props.disabled || props.loading"
		:aria-busy="props.loading || undefined"
	>
		<slot />
	</ButtonFrame>
</template>
