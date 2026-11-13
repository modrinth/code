<script setup lang="ts">
import { ChevronDownIcon } from '@modrinth/assets'
import { Accordion } from '@modrinth/ui'
import { computed } from 'vue'

import { severityOrder } from '../helpers'
import type { FlagItem, TraceVerdictEvent } from '../types'
import { injectTechReviewDecisions } from '../use-tech-review-decisions'
import TraceItem from './trace-item.vue'

const props = defineProps<{
	flags: FlagItem[]
	focusedTraceId?: string | null
	scrollable?: boolean
	canHighlight?: (value: string) => boolean
}>()
const showHiddenTraces = defineModel<boolean>('showHiddenTraces', { default: false })
const emit = defineEmits<{
	verdict: [event: TraceVerdictEvent]
	selectSourceValue: [value: string]
}>()

const { getMarkedFlagsCount } = injectTechReviewDecisions()

const traceSections = computed(() => [
	{
		id: 'traces',
		label: 'Traces',
		flags: props.flags
			.filter((flag) => flag.detail.severity !== 'hidden')
			.sort((a, b) => severityOrder[b.detail.severity] - severityOrder[a.detail.severity]),
	},
	{
		id: 'hidden',
		label: 'Hidden traces',
		flags: props.flags.filter((flag) => flag.detail.severity === 'hidden'),
	},
])
</script>

<template>
	<div data-trace-list :class="scrollable ? 'min-h-0 flex-1 overflow-y-auto px-2' : 'contents'">
		<component
			:is="section.id === 'hidden' ? Accordion : 'section'"
			v-for="(section, sectionIndex) in traceSections.filter((section) => section.flags.length)"
			:key="section.id"
			:open-by-default="section.id === 'hidden' ? showHiddenTraces : undefined"
			:button-class="
				section.id === 'hidden'
					? `custom-focus-indicator sticky top-0 z-10 flex w-full cursor-pointer items-center gap-1 border-0 bg-bg px-0 py-2 text-left text-sm font-semibold text-primary ${sectionIndex > 0 ? 'pt-5' : ''}`
					: undefined
			"
			@on-open="showHiddenTraces = true"
			@on-close="showHiddenTraces = false"
		>
			<template #button="{ open }">
				<span>
					{{ section.label }} ({{ getMarkedFlagsCount(section.flags) }}/{{ section.flags.length }})
				</span>
				<ChevronDownIcon
					class="size-4 shrink-0 transition-transform"
					:class="{ '-rotate-90': !open }"
					aria-hidden="true"
				/>
			</template>
			<div
				v-if="section.id !== 'hidden'"
				class="sticky top-0 z-10 flex flex-wrap items-center justify-between gap-2 bg-bg py-2"
			>
				<h4 class="m-0 pt-1 text-sm font-semibold text-primary">
					Traces ({{ section.flags.length }})
				</h4>
				<slot name="actions" :flags="section.flags" />
			</div>
			<TraceItem
				v-for="(flag, flagIndex) in section.flags"
				:key="`${flag.issueId}-${flag.detail.id}`"
				:flag="flag"
				:focused="String(flag.detail.id) === focusedTraceId"
				:divider="
					flagIndex < section.flags.length - 1 ||
					(section.id === 'traces' &&
						traceSections.some((item) => item.id === 'hidden' && item.flags.length > 0))
				"
				:can-highlight="canHighlight"
				@verdict="emit('verdict', $event)"
				@select-source-value="emit('selectSourceValue', $event)"
			/>
		</component>
	</div>
</template>
