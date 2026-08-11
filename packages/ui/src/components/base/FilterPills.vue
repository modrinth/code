<template>
	<div class="flex items-center gap-2">
		<FilterIcon class="size-5 shrink-0 text-secondary" />
		<div class="filter-pills__chips flex flex-wrap items-center gap-1.5">
			<button
				type="button"
				:class="pillClass(modelValue.length === 0)"
				:aria-pressed="modelValue.length === 0"
				@click="modelValue = []"
			>
				<slot name="all"> All </slot>
			</button>
			<button
				v-for="option in options"
				:key="option.id"
				type="button"
				:class="pillClass(modelValue.includes(option.id))"
				:aria-pressed="modelValue.includes(option.id)"
				@click="toggle(option.id)"
			>
				{{ option.label }}
			</button>
		</div>
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
		'cursor-pointer rounded-xl border border-solid px-3 py-1.5 text-sm font-medium leading-5 transition-all duration-100 active:scale-[0.97] focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow',
		active
			? 'border-brand bg-brand-highlight text-brand'
			: 'border-surface-5 bg-transparent text-primary hover:bg-surface-3',
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

<style scoped>
.filter-pills__chips {
	filter: drop-shadow(0 1px 1.5px rgba(0, 0, 0, 0.15));
}
</style>
