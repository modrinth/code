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
import StatCard from './StatCard.vue'

const MONETIZATION_BANNER_DISMISSED_KEY = 'analytics-monetization-banner-dismissed'

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
const { formatMessage } = useVIntl()
const monetizationBannerDismissed = useLocalStorage(MONETIZATION_BANNER_DISMISSED_KEY, false)
const showMonetizationBanner = computed(
	() => selectedBreakdowns.value.includes('monetization') && !monetizationBannerDismissed.value,
)

const compactNumberFormatter = computed(
	() =>
		new Intl.NumberFormat(undefined, {
			notation: 'compact',
			maximumSignificantDigits: 2,
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

function dismissMonetizationBanner() {
	monetizationBannerDismissed.value = true
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
		label: formatAnalyticsStatLabel('views', formatMessage),
		statLabel: formatStatNumber(currentTotals.value.views),
		vsPrevPeriodPercent: formatPreviousPeriodPercent(percentChanges.value.views),
		icon: 'eye',
		disabled: !isAnalyticsDashboardStatRelevant('views', selectedBreakdowns.value),
	},
	{
		key: 'downloads',
		label: formatAnalyticsStatLabel('downloads', formatMessage),
		statLabel: formatStatNumber(currentTotals.value.downloads),
		vsPrevPeriodPercent: formatPreviousPeriodPercent(percentChanges.value.downloads),
		icon: 'download',
		disabled: !isAnalyticsDashboardStatRelevant('downloads', selectedBreakdowns.value),
	},
	{
		key: 'revenue',
		label: formatAnalyticsStatLabel('revenue', formatMessage),
		statLabel: formatMessage(analyticsStatCardMessages.revenueValue, {
			value: formatStatNumber(currentTotals.value.revenue),
		}),
		vsPrevPeriodPercent: formatPreviousPeriodPercent(percentChanges.value.revenue),
		icon: 'dollar',
		disabled: !isAnalyticsDashboardStatRelevant('revenue', selectedBreakdowns.value),
	},
	{
		key: 'playtime',
		label: formatAnalyticsStatLabel('playtime', formatMessage),
		statLabel: formatMessage(analyticsStatCardMessages.playtimeHours, {
			hours: formatStatNumber(currentTotals.value.playtime / 3600),
		}),
		vsPrevPeriodPercent: formatPreviousPeriodPercent(percentChanges.value.playtime),
		icon: 'clock',
		disabled: !isAnalyticsDashboardStatRelevant('playtime', selectedBreakdowns.value),
	},
])
</script>
