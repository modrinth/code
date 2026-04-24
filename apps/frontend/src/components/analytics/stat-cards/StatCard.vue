<template>
	<button
		v-tooltip="disabled ? 'Stat type not appicable to breakdown' : ''"
		type="button"
		class="flex h-full appearance-none flex-col gap-4 rounded-2xl border border-solid p-4 text-left transition-colors"
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
				class="text-base font-semibold"
				:class="{
					'text-secondary': disabled,
					'text-primary': !disabled,
				}"
			>
				{{ label }}
			</div>

			<div
				class="inline-flex size-8 items-center justify-center rounded-lg border border-solid"
				:class="{
					'border-surface-5 bg-surface-3': disabled,
					'border-brand bg-brand-highlight': !disabled && active,
					'border-surface-5 bg-surface-4': !disabled && !active,
				}"
			>
				<component
					:is="iconComponent"
					aria-hidden="true"
					class="size-4"
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
				class="text-3xl font-semibold leading-none"
				:class="{
					'text-primary': disabled,
					'text-contrast': !disabled,
				}"
			>
				{{ disabled ? '-' : statLabel }}
			</div>

			<template v-if="disabled">
				<span class="inline-flex items-center gap-1 text-xs text-secondary">N/A</span>
			</template>
			<template v-else>
				<div class="flex items-center gap-1 text-sm">
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
						class="mt-px text-xs"
						:class="{
							'text-secondary': disabled,
							'text-primary': !disabled,
						}"
					>
						vs prev period
					</span>
				</div>
			</template>
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
	disabled?: boolean
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

const showTrendDirectionIcon = computed(() => !props.disabled && trendValue.value !== 0)

const trendDirectionIcon = computed<IconComponent>(() =>
	trendValue.value >= 0 ? ArrowUpRightIcon : ArrowDownLeftIcon,
)
</script>
