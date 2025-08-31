<template>
	<button
		role="radio"
		:aria-checked="selected"
		:aria-disabled="disabled"
		class="px-4 py-3 text-left border-0 font-medium border-2 border-button-bg border-solid flex gap-2 transition-all cursor-pointer rounded-xl"
		:class="
			(selected ? 'text-contrast bg-button-bg' : 'text-primary bg-transparent') +
			(disabled
				? ' opacity-50'
				: ' active:scale-[0.98] hover:bg-button-bg hover:brightness-[--hover-brightness]')
		"
		:disabled="disabled"
		@click="emit('select')"
	>
		<RadioButtonCheckedIcon
			v-if="selected"
			class="text-brand h-5 w-5 shrink-0"
			aria-hidden="true"
		/>
		<RadioButtonIcon v-else class="h-5 w-5 shrink-0" aria-hidden="true" />
		<slot />
	</button>
</template>
<script setup lang="ts" generic="T">
import { RadioButtonCheckedIcon, RadioButtonIcon } from '@modrinth/assets'

const emit = defineEmits<{
	(e: 'select'): void
}>()

withDefaults(
	defineProps<{
		selected: boolean
		disabled?: boolean
	}>(),
	{
		disabled: false,
	},
)
</script>
