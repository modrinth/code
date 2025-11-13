<template>
	<button
		class="group bg-transparent border-none p-0 m-0 flex items-center gap-3 checkbox-outer outline-offset-4 text-contrast"
		:disabled="disabled"
		:class="
			disabled
				? 'cursor-not-allowed opacity-50'
				: 'cursor-pointer hover:brightness-[--hover-brightness] focus-visible:brightness-[--hover-brightness]'
		"
		:aria-label="description"
		:aria-checked="modelValue"
		role="checkbox"
		@click="toggle"
	>
		<span
			class="w-5 h-5 rounded-md flex items-center justify-center border-[1px] border-solid"
			:class="
				(modelValue
					? 'bg-brand border-button-border text-brand-inverted'
					: 'bg-surface-2 border-surface-5') +
				(disabled ? '' : ' checkbox-shadow group-active:scale-95')
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

<style lang="scss" scoped>
.checkbox-shadow {
	box-shadow: 1px 1px 2px 0 rgba(0, 0, 0, 0.08);
}
</style>
