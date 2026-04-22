<template>
	<div class="grid grid-cols-1 gap-3 lg:grid-cols-2 xl:grid-cols-4">
		<StatCard
			v-for="card in statCards"
			:key="card.key"
			:label="card.label"
			:stat-label="card.statLabel"
			:vs-prev-period-percent="card.vsPrevPeriodPercent"
			:icon="card.icon"
			:active="analyticsDashboardContext.activeStat.value === card.key"
			@click="analyticsDashboardContext.setActiveStat(card.key)"
		/>
	</div>
</template>

<script setup lang="ts">
import { useFormatNumber } from '@modrinth/ui'

import { type AnalyticsDashboardStat, injectAnalyticsDashboardContext } from '~/providers/analytics'

import StatCard from './StatCard.vue'

const analyticsDashboardContext = injectAnalyticsDashboardContext()
const formatNumber = useFormatNumber()

const compactNumberFormatter = computed(
	() =>
		new Intl.NumberFormat(undefined, {
			notation: 'compact',
			maximumFractionDigits: 1,
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

const statCards = computed<
	{
		key: AnalyticsDashboardStat
		label: string
		statLabel: string
		vsPrevPeriodPercent: string
		icon: string
	}[]
>(() => [
	{
		key: 'views',
		label: 'Views',
		statLabel: formatStatNumber(analyticsDashboardContext.currentTotals.value.views),
		vsPrevPeriodPercent: formatPercent(analyticsDashboardContext.percentChanges.value.views),
		icon: 'eye',
	},
	{
		key: 'downloads',
		label: 'Downloads',
		statLabel: formatStatNumber(analyticsDashboardContext.currentTotals.value.downloads),
		vsPrevPeriodPercent: formatPercent(analyticsDashboardContext.percentChanges.value.downloads),
		icon: 'download',
	},
	{
		key: 'revenue',
		label: 'Revenue',
		statLabel: `$${formatStatNumber(analyticsDashboardContext.currentTotals.value.revenue)}`,
		vsPrevPeriodPercent: formatPercent(analyticsDashboardContext.percentChanges.value.revenue),
		icon: 'dollar',
	},
	{
		key: 'playtime',
		label: 'Playtime',
		statLabel: `${formatStatNumber(analyticsDashboardContext.currentTotals.value.playtime / 3600)} hrs`,
		vsPrevPeriodPercent: formatPercent(analyticsDashboardContext.percentChanges.value.playtime),
		icon: 'clock',
	},
])
</script>
