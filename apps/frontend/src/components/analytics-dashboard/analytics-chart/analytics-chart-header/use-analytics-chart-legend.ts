import { useVIntl } from '@modrinth/ui'
import { computed, type ComputedRef, type Ref, ref, watch } from 'vue'

import type {
	AnalyticsBreakdownPreset,
	AnalyticsDashboardProject,
} from '~/providers/analytics/analytics'

import { analyticsChartMessages } from '../../analytics-messages.ts'
import { COMBINED_BREAKDOWN_DATASET_ID_PREFIX } from '../../breakdown.ts'
import {
	ALL_PROJECTS_DATASET_ID,
	MONETIZATION_LEGEND_ENTRY_ORDER,
	PREVIOUS_PERIOD_BORDER_DASH,
} from '../analytics-chart-constants.ts'
import type { AnalyticsChartEvent } from '../analytics-chart-plot/AnalyticsChartEvents.vue'
import type { AnalyticsChartLegendEntry } from '../analytics-chart-types.ts'
import {
	areStringArraysEqual,
	type ChartDataset,
	decodeBreakdownDatasetValue,
	getChartDatasetTotal,
	getPreviousPeriodDatasetId,
} from '../analytics-chart-utils.ts'

export function useAnalyticsChartLegend({
	selectableChartDatasets,
	allChartDatasets,
	previousChartDatasets,
	shouldShowPreviousPeriod,
	isSameDayLastWeekComparison,
	isRatioMode,
	hiddenGraphDatasetIds,
	selectedBreakdowns,
	isGraphDatasetSelectionActive,
	selectedProjects,
	selectedProjectIdSet,
	selectedProjectEventIdSet,
}: {
	selectableChartDatasets: ComputedRef<ChartDataset[]>
	allChartDatasets: ComputedRef<ChartDataset[]>
	previousChartDatasets: ComputedRef<ChartDataset[]>
	shouldShowPreviousPeriod: ComputedRef<boolean>
	isSameDayLastWeekComparison: ComputedRef<boolean>
	isRatioMode: Ref<boolean>
	hiddenGraphDatasetIds: Ref<string[]>
	selectedBreakdowns: Ref<readonly AnalyticsBreakdownPreset[]>
	isGraphDatasetSelectionActive: Ref<boolean>
	selectedProjects: ComputedRef<AnalyticsDashboardProject[]>
	selectedProjectIdSet: ComputedRef<Set<string>>
	selectedProjectEventIdSet: ComputedRef<Set<string>>
}) {
	const { formatMessage } = useVIntl()
	const hoveredLegendEntryId = ref<string | null>(null)
	const hiddenDatasetIds = computed(() => new Set(hiddenGraphDatasetIds.value))
	const previousPeriodSuffixMessage = computed(() =>
		isSameDayLastWeekComparison.value
			? analyticsChartMessages.sameDayLastWeekSuffix
			: analyticsChartMessages.previousPeriodSuffix,
	)
	const previousChartDatasetByOriginalId = computed(() => {
		const datasets = new Map<string, ChartDataset>()
		for (const dataset of previousChartDatasets.value) {
			datasets.set(dataset.projectId, dataset)
		}
		return datasets
	})
	const currentLegendEntries = computed<AnalyticsChartLegendEntry[]>(() =>
		selectableChartDatasets.value
			.map((dataset) => ({
				id: dataset.projectId,
				name: dataset.label,
				projectName: dataset.projectName,
				tooltip: dataset.tooltip,
				color: dataset.borderColor,
				totalValue: getChartDatasetTotal(dataset),
				hidden: hiddenDatasetIds.value.has(dataset.projectId),
			}))
			.sort(compareLegendEntries),
	)
	const visibleProjectEventIdSet = computed(() => {
		if (!selectedBreakdowns.value.includes('project')) {
			return selectedProjectEventIdSet.value
		}

		const visibleProjectIds = new Set<string>()
		const projectIdsWithLegendEntries = new Set<string>()

		for (const legendEntry of currentLegendEntries.value) {
			const projectId = getLegendEntryProjectId(legendEntry)
			if (!projectId) {
				continue
			}

			projectIdsWithLegendEntries.add(projectId)
			if (!legendEntry.hidden) {
				visibleProjectIds.add(projectId)
			}
		}

		if (isGraphDatasetSelectionActive.value) {
			return visibleProjectIds
		}

		if (projectIdsWithLegendEntries.size === 0) {
			return selectedProjectEventIdSet.value
		}

		const eventProjectIds = new Set<string>()
		for (const projectId of selectedProjectEventIdSet.value) {
			if (!projectIdsWithLegendEntries.has(projectId) || visibleProjectIds.has(projectId)) {
				eventProjectIds.add(projectId)
			}
		}

		return eventProjectIds
	})
	const legendEntries = computed<AnalyticsChartLegendEntry[]>(() => {
		if (!shouldShowPreviousPeriod.value) {
			return currentLegendEntries.value
		}

		return currentLegendEntries.value.flatMap((entry) => {
			const previousDataset = previousChartDatasetByOriginalId.value.get(entry.id)
			const previousEntry: AnalyticsChartLegendEntry = {
				id: getPreviousPeriodDatasetId(entry.id),
				name: formatMessage(previousPeriodSuffixMessage.value, { name: entry.name }),
				projectName: entry.projectName,
				tooltip: entry.tooltip,
				color: entry.color,
				totalValue: previousDataset ? getChartDatasetTotal(previousDataset) : 0,
				hidden: hiddenDatasetIds.value.has(getPreviousPeriodDatasetId(entry.id)),
				isPreviousPeriod: true,
			}

			return [entry, previousEntry]
		})
	})
	const hiddenCurrentLegendEntryIds = computed(() =>
		currentLegendEntries.value.filter((entry) => entry.hidden).map((entry) => entry.id),
	)
	const hiddenCurrentLegendEntryIdsKey = computed(() =>
		hiddenCurrentLegendEntryIds.value.join('\u0000'),
	)
	const chartDatasetById = computed(() => {
		const datasets = new Map<string, ChartDataset>()
		for (const dataset of selectableChartDatasets.value) {
			datasets.set(dataset.projectId, dataset)

			if (!shouldShowPreviousPeriod.value) {
				continue
			}

			const previousDataset = previousChartDatasetByOriginalId.value.get(dataset.projectId)
			const previousData = Array.from(
				{ length: dataset.data.length },
				(_, index) => previousDataset?.data[index] ?? 0,
			)
			datasets.set(getPreviousPeriodDatasetId(dataset.projectId), {
				projectId: getPreviousPeriodDatasetId(dataset.projectId),
				label: formatMessage(previousPeriodSuffixMessage.value, {
					name: dataset.label,
				}),
				projectName: dataset.projectName,
				tooltip: dataset.tooltip,
				data: previousData,
				borderColor: dataset.borderColor,
				backgroundColor: dataset.backgroundColor,
				borderDash: PREVIOUS_PERIOD_BORDER_DASH,
			})
		}
		return datasets
	})
	const hoverRatioSliceTotals = computed(() => {
		const sliceLength = selectableChartDatasets.value.reduce(
			(maxLength, dataset) => Math.max(maxLength, dataset.data.length),
			0,
		)
		const totals = new Array<number>(sliceLength).fill(0)

		for (const legendEntry of legendEntries.value) {
			if (legendEntry.hidden) continue

			const dataset = chartDatasetById.value.get(legendEntry.id)
			if (!dataset) continue

			for (let i = 0; i < sliceLength; i++) {
				totals[i] += dataset.data[i] ?? 0
			}
		}

		return totals
	})
	const baseVisibleChartDatasets = computed(() =>
		legendEntries.value
			.filter((legendEntry) => !legendEntry.hidden)
			.map((legendEntry) => {
				const dataset = chartDatasetById.value.get(legendEntry.id)
				if (!dataset) return null

				return {
					...dataset,
					borderColor: legendEntry.color,
					backgroundColor: legendEntry.color,
				}
			})
			.filter((dataset): dataset is ChartDataset => Boolean(dataset)),
	)
	const visibleChartDatasets = computed<ChartDataset[]>(() => {
		const datasets = baseVisibleChartDatasets.value
		if (!isRatioMode.value || datasets.length === 0) return datasets

		const sliceLength = datasets.reduce(
			(maxLength, dataset) => Math.max(maxLength, dataset.data.length),
			0,
		)
		const totals = new Array<number>(sliceLength).fill(0)
		for (const dataset of datasets) {
			for (let i = 0; i < sliceLength; i++) {
				totals[i] += dataset.data[i] ?? 0
			}
		}

		return datasets.map((dataset) => ({
			...dataset,
			data: dataset.data.map((value, i) => (totals[i] === 0 ? 0 : (value / totals[i]) * 100)),
		}))
	})
	const visibleChartDatasetById = computed(() => {
		const datasets = new Map<string, ChartDataset>()
		for (const dataset of visibleChartDatasets.value) {
			datasets.set(dataset.projectId, dataset)
		}
		return datasets
	})
	const highlightedChartDatasetId = computed(() => {
		const datasetId = hoveredLegendEntryId.value
		if (!datasetId || !visibleChartDatasetById.value.has(datasetId)) return null
		return datasetId
	})

	function compareLegendEntries(a: AnalyticsChartLegendEntry, b: AnalyticsChartLegendEntry) {
		if (selectedBreakdowns.value.length === 1 && selectedBreakdowns.value[0] === 'monetization') {
			const aOrder = MONETIZATION_LEGEND_ENTRY_ORDER.get(a.id)
			const bOrder = MONETIZATION_LEGEND_ENTRY_ORDER.get(b.id)

			if (aOrder !== undefined || bOrder !== undefined) {
				return (aOrder ?? Number.MAX_SAFE_INTEGER) - (bOrder ?? Number.MAX_SAFE_INTEGER)
			}
		}

		return b.totalValue - a.totalValue || a.name.localeCompare(b.name)
	}

	function isProjectChartEventVisibleForLegend(event: AnalyticsChartEvent) {
		return !event.projectId || visibleProjectEventIdSet.value.has(event.projectId)
	}

	function getLegendEntryProjectId(legendEntry: AnalyticsChartLegendEntry) {
		const projectBreakdownIndex = selectedBreakdowns.value.findIndex(
			(breakdown) => breakdown === 'project',
		)

		if (projectBreakdownIndex === -1) {
			if (selectedProjects.value.length === 1 && legendEntry.id === ALL_PROJECTS_DATASET_ID) {
				return selectedProjects.value[0]?.id ?? null
			}

			return null
		}

		if (selectedBreakdowns.value.length === 1) {
			return selectedProjectIdSet.value.has(legendEntry.id) ? legendEntry.id : null
		}

		if (!legendEntry.id.startsWith(COMBINED_BREAKDOWN_DATASET_ID_PREFIX)) {
			return null
		}

		const values = legendEntry.id
			.slice(COMBINED_BREAKDOWN_DATASET_ID_PREFIX.length)
			.split('+')
			.map(decodeBreakdownDatasetValue)
		const projectId = values[projectBreakdownIndex]
		return projectId && selectedProjectIdSet.value.has(projectId) ? projectId : null
	}

	function hidePreviousPeriodEntriesForHiddenCurrentEntries() {
		if (hiddenCurrentLegendEntryIds.value.length === 0) return

		const nextHiddenDatasetIds = new Set(hiddenGraphDatasetIds.value)
		for (const datasetId of hiddenCurrentLegendEntryIds.value) {
			nextHiddenDatasetIds.add(getPreviousPeriodDatasetId(datasetId))
		}

		const nextHiddenDatasetIdList = Array.from(nextHiddenDatasetIds)
		if (!areStringArraysEqual(hiddenGraphDatasetIds.value, nextHiddenDatasetIdList)) {
			hiddenGraphDatasetIds.value = nextHiddenDatasetIdList
		}
	}

	function isLegendEntryToggleDisabled(legendEntry: AnalyticsChartLegendEntry) {
		if (legendEntry.hidden) return false
		const visibleCount = legendEntries.value.filter((entry) => !entry.hidden).length
		return visibleCount <= 1
	}

	function getLegendEntryTooltip(legendEntry: AnalyticsChartLegendEntry) {
		return legendEntry.tooltip ?? legendEntry.projectName ?? ''
	}

	function isUnmonetizedLegendEntry(legendEntry: AnalyticsChartLegendEntry) {
		return (
			selectedBreakdowns.value.length === 1 &&
			selectedBreakdowns.value[0] === 'monetization' &&
			legendEntry.id === 'breakdown:unmonetized'
		)
	}

	function setHoveredLegendEntryId(datasetId: string) {
		hoveredLegendEntryId.value = datasetId
	}

	function clearHoveredLegendEntryId(datasetId: string) {
		if (hoveredLegendEntryId.value === datasetId) {
			hoveredLegendEntryId.value = null
		}
	}

	function clearLegendHoverState() {
		hoveredLegendEntryId.value = null
	}

	function toggleLegendEntryVisibility(datasetId: string) {
		const nextHiddenDatasetIds = new Set(hiddenDatasetIds.value)
		if (nextHiddenDatasetIds.has(datasetId)) {
			nextHiddenDatasetIds.delete(datasetId)
		} else {
			const visibleCount = legendEntries.value.filter((entry) => !entry.hidden).length
			if (visibleCount <= 1) return
			nextHiddenDatasetIds.add(datasetId)
		}
		hiddenGraphDatasetIds.value = Array.from(nextHiddenDatasetIds)
	}

	function soloLegendEntry(datasetId: string) {
		const currentLegendEntryIds = new Set(legendEntries.value.map((entry) => entry.id))
		const otherIds = legendEntries.value.map((entry) => entry.id).filter((id) => id !== datasetId)
		const isAlreadySolo =
			!hiddenDatasetIds.value.has(datasetId) &&
			otherIds.every((id) => hiddenDatasetIds.value.has(id))

		if (isAlreadySolo) {
			hiddenGraphDatasetIds.value = hiddenGraphDatasetIds.value.filter(
				(hiddenDatasetId) => !currentLegendEntryIds.has(hiddenDatasetId),
			)
			return
		}

		const nextHiddenDatasetIds = new Set(hiddenDatasetIds.value)
		for (const legendEntry of legendEntries.value) {
			if (legendEntry.id === datasetId) {
				nextHiddenDatasetIds.delete(legendEntry.id)
			} else {
				nextHiddenDatasetIds.add(legendEntry.id)
			}
		}
		hiddenGraphDatasetIds.value = Array.from(nextHiddenDatasetIds)
	}

	function onLegendEntryClick(event: MouseEvent, datasetId: string) {
		if (event.shiftKey) {
			soloLegendEntry(datasetId)
			clearLegendHoverState()
			return
		}
		toggleLegendEntryVisibility(datasetId)
		clearLegendHoverState()
	}

	function onTooltipEntryClick(datasetId: string, shiftKey: boolean) {
		if (!chartDatasetById.value.has(datasetId)) return

		if (shiftKey) {
			soloLegendEntry(datasetId)
			clearLegendHoverState()
			return
		}
		toggleLegendEntryVisibility(datasetId)
		clearLegendHoverState()
	}

	watch(
		[shouldShowPreviousPeriod, hiddenCurrentLegendEntryIdsKey],
		([showPreviousPeriod]) => {
			if (!showPreviousPeriod) return
			hidePreviousPeriodEntriesForHiddenCurrentEntries()
		},
		{ immediate: true },
	)

	watch(
		[allChartDatasets, legendEntries],
		([datasets]) => {
			if (datasets.length === 0) return

			const availableDatasetIds = new Set(legendEntries.value.map((entry) => entry.id))
			const nextHiddenDatasetIds = hiddenGraphDatasetIds.value.filter((datasetId) =>
				availableDatasetIds.has(datasetId),
			)
			if (
				legendEntries.value.length > 0 &&
				legendEntries.value.every((entry) => nextHiddenDatasetIds.includes(entry.id))
			) {
				const firstLegendEntry = legendEntries.value[0]
				if (firstLegendEntry) {
					const firstLegendEntryIndex = nextHiddenDatasetIds.indexOf(firstLegendEntry.id)
					if (firstLegendEntryIndex !== -1) {
						nextHiddenDatasetIds.splice(firstLegendEntryIndex, 1)
					}
				}
			}

			if (!areStringArraysEqual(hiddenGraphDatasetIds.value, nextHiddenDatasetIds)) {
				hiddenGraphDatasetIds.value = nextHiddenDatasetIds
			}
		},
		{ immediate: true },
	)

	return {
		hoveredLegendEntryId,
		hiddenDatasetIds,
		previousChartDatasetByOriginalId,
		currentLegendEntries,
		visibleProjectEventIdSet,
		legendEntries,
		hiddenCurrentLegendEntryIds,
		hiddenCurrentLegendEntryIdsKey,
		chartDatasetById,
		hoverRatioSliceTotals,
		baseVisibleChartDatasets,
		visibleChartDatasets,
		visibleChartDatasetById,
		highlightedChartDatasetId,
		isProjectChartEventVisibleForLegend,
		getLegendEntryProjectId,
		hidePreviousPeriodEntriesForHiddenCurrentEntries,
		isLegendEntryToggleDisabled,
		getLegendEntryTooltip,
		isUnmonetizedLegendEntry,
		setHoveredLegendEntryId,
		clearHoveredLegendEntryId,
		clearLegendHoverState,
		toggleLegendEntryVisibility,
		soloLegendEntry,
		onLegendEntryClick,
		onTooltipEntryClick,
	}
}
