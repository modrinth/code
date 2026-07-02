<template>
	<div class="flex w-full flex-col gap-3">
		<Admonition
			v-if="showMonetizationBanner"
			type="info"
			:header="formatMessage(analyticsStatCardMessages.monetizationBannerTitle)"
			show-actions-underneath
			dismissible
			@dismiss="dismissMonetizationBanner"
		>
			<div class="text-primary">
				{{ formatMessage(analyticsStatCardMessages.monetizationBannerBody) }}
			</div>
			<template #actions>
				<ButtonStyled color="blue">
					<a href="https://modrinth.com/legal/cmp-info" target="_blank" class="w-fit !px-4">
						{{ formatMessage(analyticsStatCardMessages.monetizationBannerLearnMore) }}
						<RightArrowIcon aria-hidden="true" />
					</a>
				</ButtonStyled>
			</template>
		</Admonition>
		<div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
			<StatCard
				v-for="card in statCards"
				:key="card.key"
				:label="card.label"
				:stat-label="card.statLabel"
				:stat-tooltip="card.statTooltip"
				:vs-prev-period-percent="card.vsPrevPeriodPercent"
				:icon="card.icon"
				:active="activeStat === card.key"
				:disabled="card.disabled"
				@click="setActiveStat(card.key)"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { RightArrowIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, useFormatNumber, useVIntl } from '@modrinth/ui'
import { useLocalStorage } from '@vueuse/core'

import {
	type AnalyticsDashboardStat,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import { analyticsStatCardMessages, formatAnalyticsStatLabel } from '../analytics-messages.ts'
import { formatAnalyticsTableFullPlaytime } from '../analytics-table/analytics-table-formatting.ts'
import StatCard from './StatCard.vue'

const MONETIZATION_BANNER_DISMISSED_KEY = 'analytics-monetization-banner-dismissed'

const {
	activeStat,
	setActiveStat,
	currentTotals,
	previousTotals,
	percentChanges,
	hasPreviousPeriodComparison,
	selectedBreakdowns,
	isAnalyticsDashboardStatRelevant,
} = injectAnalyticsDashboardContext()
const formatNumber = useFormatNumber()
const { formatMessage } = useVIntl()
const monetizationBannerDismissed = useLocalStorage(MONETIZATION_BANNER_DISMISSED_KEY, false)
const showMonetizationBanner = computed(
	() => selectedBreakdowns.value.includes('monetization') && !monetizationBannerDismissed.value,
)
const MAX_PREVIOUS_PERIOD_PERCENT_DISPLAY = 1000

const compactNumberFormatter = computed(
	() =>
		new Intl.NumberFormat(undefined, {
			notation: 'compact',
			maximumSignificantDigits: 2,
		}),
)

const underDollarRevenueFormatter = computed(
	() =>
		new Intl.NumberFormat(undefined, {
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		}),
)

const preciseRevenueFormatter = computed(
	() =>
		new Intl.NumberFormat(undefined, {
			minimumFractionDigits: 5,
			maximumFractionDigits: 5,
		}),
)

const tooltipRevenueFormatter = computed(
	() =>
		new Intl.NumberFormat(undefined, {
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		}),
)

const underHourPlaytimeFormatter = computed(
	() =>
		new Intl.NumberFormat(undefined, {
			minimumFractionDigits: 2,
			maximumFractionDigits: 2,
		}),
)

function formatStatNumber(value: number): string {
	const rounded = Math.round(value)

	if (Math.abs(rounded) >= 1000) {
		return compactNumberFormatter.value.format(rounded)
	}

	return formatNumber(rounded)
}

function formatFullStatNumber(value: number): string {
	return formatNumber(Math.round(value))
}

function formatRevenueNumber(value: number): string {
	if (Math.abs(value) > 0 && Math.abs(value) < 1) {
		return underDollarRevenueFormatter.value.format(value)
	}

	return formatStatNumber(value)
}

function formatRevenueValue(value: number): string {
	return formatMessage(analyticsStatCardMessages.revenueValue, {
		value: formatRevenueNumber(value),
	})
}

function formatPreciseRevenueValue(value: number): string {
	return formatMessage(analyticsStatCardMessages.revenueValue, {
		value:
			Math.abs(value) < 1
				? preciseRevenueFormatter.value.format(value)
				: tooltipRevenueFormatter.value.format(value),
	})
}

function formatPlaytimeTooltip(value: number): string {
	return formatAnalyticsTableFullPlaytime(value, formatMessage)
}

function formatPlaytimeNumber(value: number): string {
	if (Math.abs(value) > 0 && Math.abs(value) < 1) {
		return underHourPlaytimeFormatter.value.format(value)
	}

	return formatStatNumber(value)
}

function formatPercent(value: number): string {
	const rounded = Math.round(value * 10) / 10
	if (rounded === 0) {
		return '0%'
	}

	const signPrefix = rounded > 0 ? '+' : ''
	return `${signPrefix}${rounded.toFixed(1)}%`
}

function formatSignedStatNumber(value: number): string {
	const signPrefix = value > 0 ? '+' : ''
	return `${signPrefix}${formatStatNumber(value)}`
}

function formatSignedRevenue(value: number): string {
	const signPrefix = value > 0 ? '+' : value < 0 ? '-' : ''
	return `${signPrefix}${formatMessage(analyticsStatCardMessages.revenueValue, {
		value: formatRevenueNumber(Math.abs(value)),
	})}`
}

function formatSignedPlaytimeHours(value: number): string {
	const rounded = Math.round(value * 10) / 10
	if (rounded === 0) {
		return '0'
	}

	if (Math.abs(rounded) >= 1000) {
		const signPrefix = rounded > 0 ? '+' : ''
		return `${signPrefix}${compactNumberFormatter.value.format(rounded)}`
	}

	const signPrefix = rounded > 0 ? '+' : ''
	return `${signPrefix}${rounded.toFixed(1)}`
}

function formatSignedPlaytime(value: number): string {
	return formatMessage(analyticsStatCardMessages.playtimeHours, {
		hours: formatSignedPlaytimeHours(value / 3600),
	})
}

function formatPreviousPeriodComparison(
	stat: AnalyticsDashboardStat,
	percentChange: number,
	currentValue: number,
	previousValue: number,
): string | null {
	if (!hasPreviousPeriodComparison.value) {
		return null
	}

	const delta = currentValue - previousValue
	if (previousValue === 0 && currentValue === 0) {
		return formatPercent(percentChange)
	}

	if (previousValue !== 0 && Math.abs(percentChange) <= MAX_PREVIOUS_PERIOD_PERCENT_DISPLAY) {
		return formatPercent(percentChange)
	}

	switch (stat) {
		case 'revenue':
			return formatSignedRevenue(delta)
		case 'playtime':
			return formatSignedPlaytime(delta)
		case 'views':
		case 'downloads':
			return formatSignedStatNumber(delta)
	}
}

function dismissMonetizationBanner() {
	monetizationBannerDismissed.value = true
}

const statCards = computed<
	{
		key: AnalyticsDashboardStat
		label: string
		statLabel: string
		statTooltip?: string
		vsPrevPeriodPercent: string | null
		icon: string
		disabled: boolean
	}[]
>(() => [
	{
		key: 'views',
		label: formatAnalyticsStatLabel('views', formatMessage),
		statLabel: formatStatNumber(currentTotals.value.views),
		statTooltip: formatFullStatNumber(currentTotals.value.views),
		vsPrevPeriodPercent: formatPreviousPeriodComparison(
			'views',
			percentChanges.value.views,
			currentTotals.value.views,
			previousTotals.value.views,
		),
		icon: 'eye',
		disabled: !isAnalyticsDashboardStatRelevant('views', selectedBreakdowns.value),
	},
	{
		key: 'downloads',
		label: formatAnalyticsStatLabel('downloads', formatMessage),
		statLabel: formatStatNumber(currentTotals.value.downloads),
		statTooltip: formatFullStatNumber(currentTotals.value.downloads),
		vsPrevPeriodPercent: formatPreviousPeriodComparison(
			'downloads',
			percentChanges.value.downloads,
			currentTotals.value.downloads,
			previousTotals.value.downloads,
		),
		icon: 'download',
		disabled: !isAnalyticsDashboardStatRelevant('downloads', selectedBreakdowns.value),
	},
	{
		key: 'revenue',
		label: formatAnalyticsStatLabel('revenue', formatMessage),
		statLabel: formatRevenueValue(currentTotals.value.revenue),
		statTooltip: formatPreciseRevenueValue(currentTotals.value.revenue),
		vsPrevPeriodPercent: formatPreviousPeriodComparison(
			'revenue',
			percentChanges.value.revenue,
			currentTotals.value.revenue,
			previousTotals.value.revenue,
		),
		icon: 'dollar',
		disabled: !isAnalyticsDashboardStatRelevant('revenue', selectedBreakdowns.value),
	},
	{
		key: 'playtime',
		label: formatAnalyticsStatLabel('playtime', formatMessage),
		statLabel: formatMessage(analyticsStatCardMessages.playtimeHours, {
			hours: formatPlaytimeNumber(currentTotals.value.playtime / 3600),
		}),
		statTooltip: formatPlaytimeTooltip(currentTotals.value.playtime),
		vsPrevPeriodPercent: formatPreviousPeriodComparison(
			'playtime',
			percentChanges.value.playtime,
			currentTotals.value.playtime,
			previousTotals.value.playtime,
		),
		icon: 'clock',
		disabled: !isAnalyticsDashboardStatRelevant('playtime', selectedBreakdowns.value),
	},
])
</script>
