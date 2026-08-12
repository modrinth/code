<template>
	<div class="flex flex-wrap items-center gap-1.5">
		<FilterIcon class="size-5 text-secondary" />
		<button
			:class="pillClass(modelValue.length === 0)"
			:aria-pressed="modelValue.length === 0"
			@click="modelValue = []"
		>
			<slot name="all"> All </slot>
		</button>
		<button
			v-for="option in options"
			:key="option.id"
			:class="pillClass(modelValue.includes(option.id))"
			:aria-pressed="modelValue.includes(option.id)"
			@click="toggle(option.id)"
		>
			{{ option.label }}
		</button>
	</div>
</template>

<script setup lang="ts">
import { FilterIcon } from '@modrinth/assets'

export interface FilterPillOption {
	id: string
	label: string
}

const modelValue = defineModel<string[]>({ required: true })

defineProps<{
	options: FilterPillOption[]
}>()

function pillClass(active: boolean) {
	return [
		'cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]',
		active
			? 'border-brand bg-brand-highlight text-brand'
			: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5',
	]
}

function toggle(id: string) {
	if (modelValue.value.includes(id)) {
		modelValue.value = modelValue.value.filter((f) => f !== id)
	} else {
		modelValue.value = [...modelValue.value, id]
	}
}
</script>
