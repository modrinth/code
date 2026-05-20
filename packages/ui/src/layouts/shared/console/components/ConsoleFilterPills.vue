<template>
	<FilterPills v-model="selectedFilters" :options="visibleOptions">
		<template #all> All </template>
	</FilterPills>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import FilterPills from '#ui/components/base/FilterPills.vue'

import type { ConditionalLevel } from '../composables/console-filtering'
import type { LogLevel } from '../types'

type FilterValue = LogLevel | 'all'

const ALWAYS_VISIBLE: Array<{ id: LogLevel; label: string }> = [
	{ id: 'error', label: 'Error' },
	{ id: 'warn', label: 'Warn' },
	{ id: 'info', label: 'Info' },
]

const CONDITIONAL_OPTIONS: Array<{ id: ConditionalLevel; label: string }> = [
	{ id: 'debug', label: 'Debug' },
	{ id: 'trace', label: 'Trace' },
]

const props = defineProps<{
	presentLevels: Set<ConditionalLevel>
}>()

const modelValue = defineModel<Set<FilterValue>>({ required: true })

const emit = defineEmits<{
	toggle: [value: FilterValue]
}>()

const visibleOptions = computed(() => [
	...ALWAYS_VISIBLE,
	...CONDITIONAL_OPTIONS.filter((opt) => props.presentLevels.has(opt.id)),
])

const selectedFilters = computed({
	get() {
		if (modelValue.value.has('all')) return []
		return [...modelValue.value] as string[]
	},
	set(ids: string[]) {
		if (ids.length === 0) {
			emit('toggle', 'all')
		} else {
			const current = selectedFilters.value
			const added = ids.find((id) => !current.includes(id))
			const removed = current.find((id) => !ids.includes(id))
			if (added) emit('toggle', added as FilterValue)
			if (removed) emit('toggle', removed as FilterValue)
		}
	},
})
</script>
