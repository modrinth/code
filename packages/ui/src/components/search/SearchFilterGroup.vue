<template>
	<div class="flex flex-col">
		<button
			class="flex w-full items-center gap-1 border-none bg-transparent px-2 py-1.5 text-left text-sm font-bold tracking-wide text-primary transition-colors cursor-pointer"
			@click="open = !open"
		>
			<span>{{ groupName }}</span>
			<DropdownIcon
				class="ml-auto size-4 shrink-0 transition-transform duration-200"
				:class="{ 'rotate-180': open }"
			/>
		</button>
		<div class="accordion-content" :class="{ open }">
			<div>
				<div class="flex flex-col gap-1 ml-2">
					<SearchFilterOption
						v-for="option in options"
						:key="option.id"
						:option="option"
						:included="included(option)"
						:excluded="excluded(option)"
						:supports-negative-filter="supportsNegativeFilter"
						@toggle="(o) => emit('toggle', o)"
						@toggle-exclude="(o) => emit('toggleExclude', o)"
					>
						<span
							v-if="option.icon"
							class="inline-flex items-center justify-center shrink-0 h-4 w-4"
						>
							<div v-if="typeof option.icon === 'string'" class="h-4 w-4" v-html="option.icon" />
							<component :is="option.icon" v-else class="h-4 w-4" />
						</span>
						<span class="truncate text-sm">
							{{ option.formatted_name ?? option.id }}
						</span>
					</SearchFilterOption>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { DropdownIcon } from '@modrinth/assets'
import { ref } from 'vue'

import type { FilterOption } from '../../utils/search'
import SearchFilterOption from './SearchFilterOption.vue'

defineProps<{
	groupName: string
	options: FilterOption[]
	supportsNegativeFilter: boolean
	included: (option: FilterOption) => boolean
	excluded: (option: FilterOption) => boolean
}>()

const emit = defineEmits<{
	toggle: [option: FilterOption]
	toggleExclude: [option: FilterOption]
}>()

const open = ref(false)
</script>

<style scoped>
.accordion-content {
	display: grid;
	grid-template-rows: 0fr;
	transition: grid-template-rows 0.2s ease-in-out;
}

@media (prefers-reduced-motion) {
	.accordion-content {
		transition: none !important;
	}
}

.accordion-content.open {
	grid-template-rows: 1fr;
}

.accordion-content > div {
	overflow: hidden;
}
</style>
