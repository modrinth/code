<template>
	<button
		type="button"
		class="flex h-full appearance-none flex-col gap-4 rounded-2xl border border-solid p-4 text-left transition-colors"
		:class="
			active
				? 'cursor-default border-brand bg-highlight-green'
				: 'border-surface-5 bg-surface-3 hover:bg-surface-4 active:scale-95'
		"
		@click="emit('click')"
	>
		<div class="flex items-center justify-between gap-3">
			<div class="text-sm font-semibold text-primary">
				{{ label }}
			</div>

			<div
				class="inline-flex size-8 items-center justify-center rounded-lg border border-solid"
				:class="active ? 'border-brand bg-brand-highlight' : 'border-surface-5 bg-surface-4'"
			>
				<component
					:is="iconComponent"
					aria-hidden="true"
					class="size-4"
					:class="active ? 'text-brand' : 'text-primary'"
				/>
			</div>
		</div>

		<div class="flex flex-col gap-2">
			<div class="text-3xl font-extrabold leading-none text-contrast">
				{{ statLabel }}
			</div>

			<div class="flex items-center gap-1 text-sm">
				<span class="inline-flex items-center gap-1 font-semibold" :class="vsPrevPeriodClass">
					<component
						:is="trendDirectionIcon"
						v-if="showTrendDirectionIcon"
						aria-hidden="true"
						class="size-3"
					/>
					{{ vsPrevPeriodPercent }}
				</span>
				<span class="mt-px text-xs text-primary">vs prev period</span>
			</div>
		</div>
	</button>
</template>

<script setup lang="ts">
import {
	ArrowDownLeftIcon,
	ArrowUpRightIcon,
	ClockIcon,
	CurrencyIcon,
	DownloadIcon,
	EyeIcon,
	type IconComponent,
	PlayIcon,
	TimerIcon,
} from '@modrinth/assets'

const props = defineProps<{
	label: string
	statLabel: string
	vsPrevPeriodPercent: string
	icon: string
	active?: boolean
}>()

const emit = defineEmits<{
	(event: 'click'): void
}>()

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
	const parsed = Number.parseFloat(props.vsPrevPeriodPercent.replace(/[^0-9.-]/gu, ''))
	return Number.isNaN(parsed) ? 0 : parsed
})

const showTrendDirectionIcon = computed(() => trendValue.value !== 0)

const trendDirectionIcon = computed<IconComponent>(() =>
	trendValue.value >= 0 ? ArrowUpRightIcon : ArrowDownLeftIcon,
)

const vsPrevPeriodClass = computed(() => {
	if (trendValue.value > 0) {
		return 'text-green'
	}

	if (trendValue.value < 0) {
		return 'text-red'
	}

	return 'text-primary'
})
</script>
