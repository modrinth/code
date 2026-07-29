<template>
	<div class="relative">
		<Transition
			enter-active-class="transition-all duration-200 ease-out"
			enter-from-class="opacity-0 max-h-0"
			enter-to-class="opacity-100 max-h-5"
			leave-active-class="transition-all duration-200 ease-in"
			leave-from-class="opacity-100 max-h-5"
			leave-to-class="opacity-0 max-h-0"
		>
			<div
				v-if="showLegendTopFade"
				class="z-1 pointer-events-none absolute left-0 right-0 top-0 h-5 bg-gradient-to-b from-surface-3 to-transparent"
			/>
		</Transition>

		<div
			ref="legendContainer"
			class="flex max-h-[130px] flex-wrap items-center gap-y-1 overflow-y-auto px-3"
			@scroll="checkLegendScrollState"
		>
			<div
				v-for="legendEntry in legendEntries"
				:key="legendEntry.id"
				class="inline-flex items-center"
			>
				<button
					v-tooltip="legendEntry.tooltip ?? legendEntry.projectName ?? ''"
					type="button"
					class="inline-flex items-center gap-1.5 px-2 py-0.5 text-sm !outline-0 transition-all focus-within:!outline-0 focus:!outline-0 focus-visible:!outline-0"
					:class="[
						legendEntry.hidden ? 'text-secondary opacity-70' : 'text-primary',
						isLegendEntryToggleDisabled(legendEntry) && !isShiftKeyPressed
							? 'cursor-default'
							: 'cursor-pointer hover:brightness-125',
					]"
					:aria-pressed="!legendEntry.hidden"
					@mouseenter="emit('entry-hover', legendEntry.id)"
					@mouseleave="emit('entry-hover-clear', legendEntry.id)"
					@focus="emit('entry-hover', legendEntry.id)"
					@blur="emit('entry-hover-clear', legendEntry.id)"
					@click="emit('entry-click', $event, legendEntry.id)"
				>
					<span
						:class="
							legendEntry.isPreviousPeriod
								? 'h-0 w-2 rounded-none border-0 border-t-2 border-dashed bg-transparent'
								: 'size-2 rounded-full'
						"
						:style="
							legendEntry.isPreviousPeriod
								? { borderColor: legendEntry.color }
								: { backgroundColor: legendEntry.color }
						"
					/>
					<span
						:class="{
							'line-through': legendEntry.hidden,
							capitalize: shouldCapitalizeDatasetLabels,
						}"
					>
						{{ legendEntry.name }}
					</span>
				</button>
				<Dropdown
					v-if="showUnmonetizedInfo && legendEntry.id === 'breakdown:unmonetized'"
					theme="analytics-monetization-popover"
					:triggers="['hover', 'focus']"
					:popper-triggers="['hover', 'focus']"
					:delay="{ show: 0, hide: 250 }"
					placement="top"
					:aria-id="monetizationPopoverId"
					no-auto-focus
				>
					<InfoIcon
						class="-ml-1 mt-px inline-flex size-4 items-center justify-center rounded-full border-0 bg-transparent p-0 text-secondary transition-all hover:text-contrast focus-visible:text-contrast"
						:aria-label="formatMessage(analyticsChartMessages.viewMonetizedAnalyticsDetails)"
					/>
					<template #popper>
						<div
							role="dialog"
							:aria-label="formatMessage(analyticsChartMessages.monetizedAnalyticsDetails)"
							class="font-base w-[292px] rounded-xl border border-solid border-surface-5 bg-surface-3 p-3 text-sm leading-snug shadow-2xl"
						>
							{{ formatMessage(analyticsChartMessages.monetizedAnalyticsDetailsDescription) }}
						</div>
					</template>
				</Dropdown>
			</div>
		</div>

		<Transition
			enter-active-class="transition-all duration-200 ease-out"
			enter-from-class="opacity-0 max-h-0"
			enter-to-class="opacity-100 max-h-5"
			leave-active-class="transition-all duration-200 ease-in"
			leave-from-class="opacity-100 max-h-5"
			leave-to-class="opacity-0 max-h-0"
		>
			<div
				v-if="showLegendBottomFade"
				class="z-1 pointer-events-none absolute bottom-0 left-0 right-0 h-5 bg-gradient-to-t from-surface-3 to-transparent"
			/>
		</Transition>
	</div>
</template>

<script setup lang="ts">
import { InfoIcon } from '@modrinth/assets'
import { useScrollIndicator, useVIntl } from '@modrinth/ui'
import { Dropdown } from 'floating-vue'

import { analyticsChartMessages } from '../../analytics-messages'
import type { AnalyticsChartLegendEntry } from '../analytics-chart-types'

const props = defineProps<{
	legendEntries: AnalyticsChartLegendEntry[]
	shouldCapitalizeDatasetLabels: boolean
	showUnmonetizedInfo: boolean
}>()

const emit = defineEmits<{
	'entry-hover': [datasetId: string]
	'entry-hover-clear': [datasetId: string]
	'entry-click': [event: MouseEvent, datasetId: string]
}>()

const monetizationPopoverId = useId()
const legendContainer = ref<HTMLElement | null>(null)
const isShiftKeyPressed = ref(false)
const { formatMessage } = useVIntl()
const {
	showTopFade: showLegendTopFade,
	showBottomFade: showLegendBottomFade,
	checkScrollState: checkLegendScrollState,
	forceCheck: forceCheckLegendScrollState,
} = useScrollIndicator(legendContainer)

function updateShiftKeyState(event: KeyboardEvent) {
	isShiftKeyPressed.value = event.shiftKey
}

function clearShiftKeyState() {
	isShiftKeyPressed.value = false
}

function isLegendEntryToggleDisabled(legendEntry: AnalyticsChartLegendEntry) {
	if (legendEntry.hidden) return false
	const visibleCount = props.legendEntries.filter((entry) => !entry.hidden).length
	return visibleCount <= 1
}

watch(
	() => props.legendEntries,
	() => {
		nextTick(() => {
			forceCheckLegendScrollState()
		})
	},
	{ immediate: true, flush: 'post' },
)

onMounted(() => {
	window.addEventListener('keydown', updateShiftKeyState)
	window.addEventListener('keyup', updateShiftKeyState)
	window.addEventListener('blur', clearShiftKeyState)
})

onBeforeUnmount(() => {
	window.removeEventListener('keydown', updateShiftKeyState)
	window.removeEventListener('keyup', updateShiftKeyState)
	window.removeEventListener('blur', clearShiftKeyState)
})
</script>

<style>
.v-popper--theme-analytics-monetization-popover .v-popper__inner {
	overflow: visible !important;
	background: transparent !important;
	padding: 0 !important;
	border: 0 !important;
	box-shadow: none !important;
}

.v-popper--theme-analytics-monetization-popover .v-popper__arrow-container {
	display: none;
}
</style>
