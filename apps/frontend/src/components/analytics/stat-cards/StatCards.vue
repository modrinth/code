<template>
	<div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
		<StatCard
			v-for="card in statCards"
			:key="card.key"
			:label="card.label"
			:stat-label="card.statLabel"
			:vs-prev-period-percent="card.vsPrevPeriodPercent"
			:icon="card.icon"
			:active="activeStat === card.key"
			:disabled="card.disabled"
			@click="setActiveStat(card.key)"
		/>
	</div>
</template>

<script setup lang="ts">
import { useFormatNumber } from '@modrinth/ui'

import {
	type AnalyticsDashboardStat,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import StatCard from './StatCard.vue'

const {
	activeStat,
	setActiveStat,
	currentTotals,
	percentChanges,
	hasPreviousPeriodComparison,
	selectedBreakdowns,
	isAnalyticsDashboardStatRelevant,
} = injectAnalyticsDashboardContext()
const formatNumber = useFormatNumber()

const compactNumberFormatter = computed(
	() =>
		new Intl.NumberFormat(undefined, {
			notation: 'compact',
			maximumSignificantDigits: 3,
		}),
)

function formatStatNumber(value: number): string {
	const rounded = Math.round(value)

	if (Math.abs(rounded) >= 1000) {
		return compactNumberFormatter.value.format(rounded)
	}

	return formatNumber(rounded)
}

function formatPercent(value: number): string {
	const rounded = Math.round(value * 10) / 10
	const signPrefix = rounded > 0 ? '+' : ''
	return `${signPrefix}${rounded.toFixed(1)}%`
}

function formatPreviousPeriodPercent(value: number): string | null {
	if (!hasPreviousPeriodComparison.value) {
		return null
	}

	return formatPercent(value)
}

const statCards = computed<
	{
		key: AnalyticsDashboardStat
		label: string
		statLabel: string
		vsPrevPeriodPercent: string | null
		icon: string
		disabled: boolean
	}[]
>(() => [
	{
		key: 'views',
		label: 'Views',
		statLabel: formatStatNumber(currentTotals.value.views),
		vsPrevPeriodPercent: formatPreviousPeriodPercent(percentChanges.value.views),
		icon: 'eye',
		disabled: !isAnalyticsDashboardStatRelevant('views', selectedBreakdowns.value),
	},
	{
		key: 'downloads',
		label: 'Downloads',
		statLabel: formatStatNumber(currentTotals.value.downloads),
		vsPrevPeriodPercent: formatPreviousPeriodPercent(percentChanges.value.downloads),
		icon: 'download',
		disabled: !isAnalyticsDashboardStatRelevant('downloads', selectedBreakdowns.value),
	},
	{
		key: 'revenue',
		label: 'Revenue',
		statLabel: `$${formatStatNumber(currentTotals.value.revenue)}`,
		vsPrevPeriodPercent: formatPreviousPeriodPercent(percentChanges.value.revenue),
		icon: 'dollar',
		disabled: !isAnalyticsDashboardStatRelevant('revenue', selectedBreakdowns.value),
	},
	{
		key: 'playtime',
		label: 'Playtime',
		statLabel: `${formatStatNumber(currentTotals.value.playtime / 3600)} hrs`,
		vsPrevPeriodPercent: formatPreviousPeriodPercent(percentChanges.value.playtime),
		icon: 'clock',
		disabled: !isAnalyticsDashboardStatRelevant('playtime', selectedBreakdowns.value),
	},
])
</script>
