<template>
	<button
		v-tooltip="disabled ? formatMessage(analyticsStatCardMessages.unavailableTooltip) : ''"
		type="button"
		class="flex h-full appearance-none flex-col gap-2.5 rounded-2xl border border-solid p-5 px-4 text-left transition-colors sm:gap-4"
		:class="{
			'cursor-not-allowed border-surface-5 bg-surface-2 opacity-60': disabled,
			'cursor-default border-brand bg-highlight-green': !disabled && active,
			'border-surface-5 bg-surface-3 hover:bg-surface-4 active:scale-95': !disabled && !active,
		}"
		:disabled="disabled"
		@click="emit('click')"
	>
		<div class="flex items-center justify-between gap-3">
			<div
				class="text-base font-medium"
				:class="{
					'text-secondary': disabled,
					'text-primary': !disabled,
				}"
			>
				{{ label }}
			</div>

			<div class="inline-flex items-center justify-center">
				<component
					:is="iconComponent"
					aria-hidden="true"
					class="size-5 sm:size-6"
					:class="{
						'text-secondary': disabled,
						'text-brand': !disabled && active,
						'text-primary': !disabled && !active,
					}"
				/>
			</div>
		</div>

		<div class="flex flex-col gap-2.5">
			<div
				v-tooltip="!disabled ? statTooltip : undefined"
				class="w-fit text-2xl font-semibold leading-none md:text-4xl"
				:class="{
					'text-primary': disabled,
					'text-contrast': !disabled,
				}"
			>
				{{ disabled ? '-' : statLabel }}
			</div>

			<template v-if="disabled">
				<span class="inline-flex items-center gap-1 text-xs text-secondary">
					{{ formatMessage(analyticsStatCardMessages.unavailableLabel) }}
				</span>
			</template>
			<template v-else>
				<div v-if="vsPrevPeriodPercent" class="flex flex-wrap items-center gap-x-1 text-sm">
					<span
						class="inline-flex items-center gap-1 font-semibold"
						:class="{
							'text-secondary': disabled,
							'text-green': !disabled && trendValue > 0,
							'text-red': !disabled && trendValue < 0,
							'text-primary': !disabled && trendValue === 0,
						}"
					>
						<component
							:is="trendDirectionIcon"
							v-if="showTrendDirectionIcon"
							aria-hidden="true"
							class="size-3"
						/>
						{{ vsPrevPeriodPercent }}
					</span>
					<span
						class="mt-px text-xs max-sm:hidden"
						:class="{
							'text-secondary': disabled,
							'text-primary': !disabled,
						}"
					>
						{{ formatMessage(comparisonMessage) }}
					</span>
					<span
						class="visible mt-px text-xs sm:hidden"
						:class="{
							'text-secondary': disabled,
							'text-primary': !disabled,
						}"
					>
						{{ formatMessage(comparisonMessageShort) }}
					</span>
				</div>
			</template>
		</div>
	</button>
</template>

<script setup lang="ts">
import {
	ClockIcon,
	CurrencyIcon,
	DownloadIcon,
	EyeIcon,
	type IconComponent,
	PlayIcon,
	TimerIcon,
	TrendingDownIcon,
	TrendingUpIcon,
} from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'

import { analyticsStatCardMessages } from '../analytics-messages'

const props = defineProps<{
	label: string
	statLabel: string
	statTooltip?: string
	vsPrevPeriodPercent: string | null
	isSameDayLastWeekComparison?: boolean
	icon: string
	active?: boolean
	disabled?: boolean
}>()

const emit = defineEmits<{
	(event: 'click'): void
}>()

const { formatMessage } = useVIntl()
const comparisonMessage = computed(() =>
	props.isSameDayLastWeekComparison
		? analyticsStatCardMessages.sameDayLastWeekComparison
		: analyticsStatCardMessages.previousPeriodComparison,
)
const comparisonMessageShort = computed(() =>
	props.isSameDayLastWeekComparison
		? analyticsStatCardMessages.sameDayLastWeekComparisonShort
		: analyticsStatCardMessages.previousPeriodComparisonShort,
)
const statCardIconMap: Record<string, IconComponent> = {
	clock: ClockIcon,
	timer: TimerIcon,
	play: PlayIcon,
	eye: EyeIcon,
	download: DownloadIcon,
	currency: CurrencyIcon,
	dollar: CurrencyIcon,
}

const iconComponent = computed<IconComponent>(() => {
	const normalizedIconName = props.icon
		.toLowerCase()
		.replace(/icon$/u, '')
		.replace(/[^a-z]/gu, '')
	return statCardIconMap[normalizedIconName] ?? ClockIcon
})

const trendValue = computed(() => {
	const parsed = Number.parseFloat(props.vsPrevPeriodPercent?.replace(/[^0-9.-]/gu, '') ?? '')
	return Number.isNaN(parsed) ? 0 : parsed
})

const showTrendDirectionIcon = computed(() => !props.disabled && trendValue.value !== 0)

const trendDirectionIcon = computed<IconComponent>(() =>
	trendValue.value >= 0 ? TrendingUpIcon : TrendingDownIcon,
)
</script>
