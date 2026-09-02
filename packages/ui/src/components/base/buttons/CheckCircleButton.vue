<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import ButtonFrame from './ButtonFrame.vue'
import type { ButtonNativeType } from './types'

withDefaults(
	defineProps<{
		checked: boolean
		disabled?: boolean
		nativeType?: ButtonNativeType
	}>(),
	{
		disabled: false,
		nativeType: 'button',
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
		type="quiet"
		size="lg"
		interaction="none"
		:disabled="disabled"
		:native-type="nativeType"
		role="radio"
		:aria-checked="checked"
		class="w-full !justify-between !gap-4 !whitespace-normal !border !border-solid !px-2 !text-left !transition-colors"
		:class="
			checked
				? '!border-brand !bg-brand-highlight !text-contrast'
				: '!border-transparent !bg-transparent !text-contrast enabled:hover:!bg-surface-3'
		"
	>
		<span class="flex min-w-0 flex-1 items-center gap-2">
			<slot />
		</span>
		<span
			v-if="checked"
			class="flex size-6 shrink-0 items-center justify-center rounded-full bg-brand text-brand-inverted"
		>
			<CheckIcon class="size-4" aria-hidden="true" />
		</span>
		<span v-else class="size-6 shrink-0 rounded-full border border-solid border-surface-5" />
	</ButtonFrame>
</template>
