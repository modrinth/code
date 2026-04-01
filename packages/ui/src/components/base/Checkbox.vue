<template>
	<button
		class="group bg-transparent border-none p-0 m-0 flex items-center gap-2 checkbox-outer outline-offset-4 text-contrast"
		:disabled="disabled"
		:class="disabled ? 'cursor-not-allowed opacity-50' : 'cursor-pointer'"
		:aria-label="description || label"
		:aria-checked="modelValue"
		role="checkbox"
		@click="toggle"
	>
		<span
			class="w-4 h-4 flex rounded-[2px] items-center justify-center border-[1px] border-solid"
			:class="
				(modelValue
					? 'bg-brand border-transparent text-brand-inverted'
					: 'bg-surface-2 border-[#888888]') + (disabled ? '' : '')
			"
		>
			<MinusIcon v-if="indeterminate" aria-hidden="true" stroke-width="3" />
			<CheckIcon v-else-if="modelValue" aria-hidden="true" stroke-width="3" />
		</span>
		<!-- aria-hidden is set so screenreaders only use the <button>'s aria-label -->
		<span v-if="label" aria-hidden="true">
			{{ label }}
		</span>
		<slot v-else />
	</button>
</template>
<script setup lang="ts">
import { CheckIcon, MinusIcon } from '@modrinth/assets'

const emit = defineEmits<{
	'update:modelValue': [boolean]
}>()

const props = withDefaults(
	defineProps<{
		label?: string
		disabled?: boolean
		description?: string
		modelValue: boolean
		clickEvent?: () => void
		indeterminate?: boolean
	}>(),
	{
		label: '',
		disabled: false,
		description: '',
		modelValue: false,
		clickEvent: () => {},
		indeterminate: false,
	},
)

function toggle() {
	if (!props.disabled) {
		emit('update:modelValue', !props.modelValue)
	}
}
</script>
