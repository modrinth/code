<template>
	<div class="flex w-full flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
		<div
			class="flex min-h-[84px] w-full flex-col items-start justify-between gap-3 rounded-t-2xl border-0 border-b border-solid border-surface-5 bg-surface-3 p-4 sm:flex-row sm:items-center"
		>
			<div class="flex flex-col gap-0.5">
				<div class="w-max text-xl font-semibold text-contrast">
					{{ graphTitle }}
				</div>
				<div
					v-if="showTableSelectionSubheading"
					class="m-0 flex w-max flex-wrap items-center gap-2 text-sm text-secondary"
				>
					<span>{{ tableSelectionSubheading }}</span>

					<button
						v-if="showGraphRenderLimitButton"
						type="button"
						class="font-base border-0 bg-transparent p-0 text-sm underline transition-all hover:brightness-125"
						@click="emit('toggle-graph-render-limit', $event)"
					>
						{{ graphRenderLimitButtonLabel }}
					</button>
					<button
						v-if="showTopGraphDatasetsButton"
						type="button"
						class="font-base border-0 bg-transparent p-0 text-sm underline transition-all hover:brightness-125"
						@click="emit('show-top-graph-datasets')"
					>
						{{ formatMessage(analyticsChartMessages.showTopEight) }}
					</button>
				</div>
			</div>

			<div class="flex grow select-none flex-wrap-reverse items-center justify-end gap-2 gap-y-2">
				<AnalyticsChartControls
					v-model:ratio-mode="ratioMode"
					v-model:show-chart-events="showChartEvents"
					v-model:show-project-events="showProjectEvents"
					v-model:show-previous-period="showPreviousPeriod"
					:can-use-ratio-mode="canUseRatioMode"
					:can-show-previous-period="canShowPreviousPeriod"
					:has-chart-events="hasChartEvents"
					:has-project-events="hasProjectEvents"
					:small-toggles="smallToggles"
					:default-ratio-mode="DEFAULT_ANALYTICS_GRAPH_RATIO_MODE"
					:default-show-chart-events="DEFAULT_ANALYTICS_GRAPH_EVENTS_VISIBILITY"
					:default-show-project-events="defaultShowProjectEvents"
					:default-show-previous-period="DEFAULT_ANALYTICS_GRAPH_PREVIOUS_PERIOD_VISIBILITY"
				/>
				<Tabs
					:value="activeGraphViewMode"
					:tabs="viewModeTabs"
					@update:value="activeGraphViewMode = $event as AnalyticsGraphViewMode"
				/>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChartAreaIcon, ChartColumnBigIcon, ChartSplineIcon } from '@modrinth/assets'
import { Tabs, type TabsTab, useVIntl } from '@modrinth/ui'

import type { AnalyticsGraphViewMode } from '~/providers/analytics/analytics'
import {
	DEFAULT_ANALYTICS_GRAPH_EVENTS_VISIBILITY,
	DEFAULT_ANALYTICS_GRAPH_PREVIOUS_PERIOD_VISIBILITY,
	DEFAULT_ANALYTICS_GRAPH_RATIO_MODE,
} from '~/providers/analytics/query-builder-url'

import { analyticsChartMessages } from '../../analytics-messages.ts'
import AnalyticsChartControls from './AnalyticsChartControls.vue'

const activeGraphViewMode = defineModel<AnalyticsGraphViewMode>('activeGraphViewMode', {
	required: true,
})
const ratioMode = defineModel<boolean>('ratioMode', { required: true })
const showChartEvents = defineModel<boolean>('showChartEvents', { required: true })
const showProjectEvents = defineModel<boolean>('showProjectEvents', { required: true })
const showPreviousPeriod = defineModel<boolean>('showPreviousPeriod', { required: true })

const props = defineProps<{
	graphTitle: string
	showTableSelectionSubheading: boolean
	tableSelectionSubheading: string
	showGraphRenderLimitButton: boolean
	graphRenderLimitButtonLabel: string
	showTopGraphDatasetsButton: boolean
	canUseRatioMode: boolean
	canShowPreviousPeriod: boolean
	hasChartEvents: boolean
	hasProjectEvents: boolean
	smallToggles: boolean
	defaultShowProjectEvents: boolean
	isMobileLayout: boolean
}>()

const { formatMessage } = useVIntl()
const emit = defineEmits<{
	'toggle-graph-render-limit': [event: MouseEvent]
	'show-top-graph-datasets': []
}>()

const viewModeTabs = computed<TabsTab[]>(() => [
	{
		value: 'line',
		label: props.isMobileLayout ? '' : formatMessage(analyticsChartMessages.lineView),
		icon: ChartSplineIcon,
	},
	{
		value: 'area',
		label: props.isMobileLayout ? '' : formatMessage(analyticsChartMessages.areaView),
		icon: ChartAreaIcon,
	},
	{
		value: 'bar',
		label: props.isMobileLayout ? '' : formatMessage(analyticsChartMessages.barView),
		icon: ChartColumnBigIcon,
	},
])
</script>
