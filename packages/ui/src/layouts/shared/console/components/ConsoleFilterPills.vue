<template>
	<FilterPills v-model="selectedFilters" :options="visibleOptions">
		<template #all>{{ formatMessage(commonMessages.consoleFilterAllLevels) }}</template>
	</FilterPills>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import FilterPills from '#ui/components/base/FilterPills.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import type { ConditionalLevel } from '../composables/console-filtering'
import type { LogLevel } from '../types'

type FilterValue = LogLevel | 'all'

const { formatMessage } = useVIntl()

const logLevelLabels = defineMessages({
	error: {
		id: 'servers.console.filter.log-level.error',
		defaultMessage: 'Error',
	},
	warn: {
		id: 'servers.console.filter.log-level.warn',
		defaultMessage: 'Warn',
	},
	info: {
		id: 'servers.console.filter.log-level.info',
		defaultMessage: 'Info',
	},
	debug: {
		id: 'servers.console.filter.log-level.debug',
		defaultMessage: 'Debug',
	},
	trace: {
		id: 'servers.console.filter.log-level.trace',
		defaultMessage: 'Trace',
	},
})

const ALWAYS_VISIBLE: LogLevel[] = ['error', 'warn', 'info']

const CONDITIONAL_LEVELS: ConditionalLevel[] = ['debug', 'trace']

const props = defineProps<{
	presentLevels: Set<ConditionalLevel>
}>()

const modelValue = defineModel<Set<FilterValue>>({ required: true })

const emit = defineEmits<{
	toggle: [value: FilterValue]
}>()

const visibleOptions = computed(() => [
	...ALWAYS_VISIBLE.map((id) => ({ id, label: formatMessage(logLevelLabels[id]) })),
	...CONDITIONAL_LEVELS.filter((id) => props.presentLevels.has(id)).map((id) => ({
		id,
		label: formatMessage(logLevelLabels[id]),
	})),
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
