<template>
	<div class="search-filter-option group flex gap-1 items-center">
		<button
			:class="`flex border-none cursor-pointer !w-full items-center gap-2 truncate rounded-xl px-2 py-2 [@media(hover:hover)]:py-1 text-sm font-semibold transition-all hover:text-contrast focus-visible:text-contrast active:scale-[0.98] ${included ? 'bg-brand-highlight text-contrast hover:brightness-125' : excluded ? 'bg-highlight-red text-contrast hover:brightness-125' : 'bg-transparent text-secondary hover:bg-button-bg focus-visible:bg-button-bg [&>svg.check-icon]:hover:text-brand [&>svg.check-icon]:focus-visible:text-brand [&>svg.ban-icon]:hover:text-red [&>svg.ban-icon]:focus-visible:text-red'}`"
			@click="() => emit(primaryAction === 'exclude' ? 'toggleExclude' : 'toggle', option)"
		>
			<slot> </slot>
			<BanIcon
				v-if="excluded || primaryAction === 'exclude'"
				:class="`filter-action-icon ban-icon ml-auto h-4 w-4 shrink-0 transition-opacity group-hover:opacity-100 ${excluded ? '' : '[@media(hover:hover)]:opacity-0'}`"
				aria-hidden="true"
			/>
			<CheckIcon
				v-else
				:class="`filter-action-icon check-icon ml-auto h-4 w-4 shrink-0 transition-opacity group-hover:opacity-100 ${included ? '' : '[@media(hover:hover)]:opacity-0'}`"
				aria-hidden="true"
			/>
		</button>
		<div
			v-if="showExcludeButton"
			class="w-px h-[1.75rem] bg-button-bg [@media(hover:hover)]:contents"
			:class="{ 'opacity-0': included }"
		></div>
		<button
			v-if="showExcludeButton"
			v-tooltip="formatMessage(messages.excludeTooltip)"
			class="flex border-none cursor-pointer items-center justify-center gap-2 rounded-xl bg-transparent px-2 py-1 text-sm font-semibold text-secondary [@media(hover:hover)]:opacity-0 transition-all hover:bg-button-bg hover:text-red focus-visible:bg-button-bg focus-visible:text-red active:scale-[0.96]"
			@click="() => emit('toggleExclude', option)"
		>
			<BanIcon class="filter-action-icon h-4 w-4" aria-hidden="true" />
		</button>
	</div>
</template>

<script setup lang="ts">
import { BanIcon, CheckIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import type { FilterMode, FilterOption } from '../../utils/search'

const props = withDefaults(
	defineProps<{
		option: FilterOption
		included: boolean
		excluded: boolean
		supports?: FilterMode[]
	}>(),
	{
		supports: () => ['include'],
	},
)

const supportsInclude = computed(() => props.supports.includes('include'))
const supportsExclude = computed(() => props.supports.includes('exclude'))
const primaryAction = computed<FilterMode>(() => (supportsInclude.value ? 'include' : 'exclude'))
const showExcludeButton = computed(
	() => supportsInclude.value && supportsExclude.value && !props.excluded,
)

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	toggle: [option: FilterOption]
	toggleExclude: [option: FilterOption]
}>()

const messages = defineMessages({
	excludeTooltip: {
		id: 'search.filter.option.exclusion.add.tooltip',
		defaultMessage: 'Exclude',
	},
})
</script>
<style scoped lang="scss">
.search-filter-option:hover,
.search-filter-option:has(button:focus-visible) {
	button,
	.filter-action-icon {
		opacity: 1;
	}
}
</style>
