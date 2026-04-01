<template>
	<button
		type="button"
		class="search-filter-option group bg-transparent border-none p-0 m-0 flex w-full max-w-full items-center gap-2 px-2 py-0.5 text-left outline-offset-4 checkbox-outer cursor-pointer text-contrast"
		:aria-label="ariaLabel"
		:aria-checked="ariaChecked"
		role="checkbox"
		@click="emit('toggle', option)"
	>
		<span
			class="w-4 h-4 flex rounded-[2px] items-center justify-center border-[1px] border-solid shrink-0"
			:class="boxClass"
		>
			<BanIcon v-if="excluded" aria-hidden="true" stroke-width="3" />
			<CheckIcon v-else-if="included" aria-hidden="true" stroke-width="3" />
		</span>
		<span
			class="flex min-w-0 flex-1 items-center gap-2 truncate text-sm"
			:class="labelClass"
			aria-hidden="true"
		>
			<slot> </slot>
		</span>
	</button>
</template>

<script setup lang="ts">
import { BanIcon, CheckIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import type { FilterOption } from '../../utils/search'

const props = withDefaults(
	defineProps<{
		option: FilterOption
		included: boolean
		excluded: boolean
		supportsNegativeFilter?: boolean
	}>(),
	{
		supportsNegativeFilter: false,
	},
)

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	toggle: [option: FilterOption]
}>()

const optionName = computed(() => props.option.formatted_name ?? props.option.id)

const ariaLabel = computed(() => {
	if (props.excluded) {
		return formatMessage(messages.stateExcluded, { name: optionName.value })
	}
	if (props.included) {
		return formatMessage(messages.stateIncluded, { name: optionName.value })
	}
	return formatMessage(messages.stateOff, { name: optionName.value })
})

const ariaChecked = computed(() => {
	if (props.supportsNegativeFilter) {
		if (props.excluded) {
			return 'mixed' as const
		}
		return props.included
	}
	return props.included
})

const boxClass = computed(() => {
	if (props.excluded) {
		return 'bg-highlight-red border-transparent text-contrast'
	}
	if (props.included) {
		return 'bg-brand border-transparent text-brand-inverted'
	}
	return 'bg-surface-2 border-[#888888]'
})

const labelClass = computed(() => {
	if (props.excluded) {
		return 'text-brand-red'
	}
	if (props.included) {
		return 'text-contrast'
	}
	return 'text-secondary group-hover:text-contrast'
})

const messages = defineMessages({
	stateExcluded: {
		id: 'search.filter.option.aria.excluded',
		defaultMessage: '{name}, excluded from search',
	},
	stateIncluded: {
		id: 'search.filter.option.aria.included',
		defaultMessage: '{name}, included in search',
	},
	stateOff: {
		id: 'search.filter.option.aria.off',
		defaultMessage: '{name}, not used as filter',
	},
})
</script>
