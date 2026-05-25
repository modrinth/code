<script setup lang="ts">
import { CalendarIcon, ExternalIcon, UsersIcon } from '@modrinth/assets'
import { ProgressBar } from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed } from 'vue'

const props = defineProps<{
	goal_amount: number
	live_amount: number
}>()

const currencyFormatter = new Intl.NumberFormat('en-US', {
	style: 'currency',
	currency: 'USD',
	maximumFractionDigits: 0,
})

const formattedGoalAmount = computed(() => currencyFormatter.format(props.goal_amount))
const formattedLiveAmount = computed(() => currencyFormatter.format(props.live_amount))
const progressLabel = computed(
	() => `${formattedLiveAmount.value} of ${formattedGoalAmount.value} raised`,
)

async function openPrideFundraiser() {
	await openUrl('https://modrinth.com/pride')
}
</script>

<template>
	<section
		class="flex w-full flex-col gap-3 rounded-xl border border-solid border-surface-5 bg-button-bg p-3 text-primary"
	>
		<div class="flex w-full items-center justify-between gap-2">
			<h2 class="m-0 min-w-0 truncate text-base font-semibold text-contrast">
				Pride Fundraiser 2026
			</h2>
			<button
				type="button"
				class="m-0 flex size-5 shrink-0 cursor-pointer items-center justify-center border-0 bg-transparent p-0 text-primary transition-colors hover:text-contrast focus-visible:text-contrast"
				aria-label="Open Pride fundraiser"
				@click="openPrideFundraiser"
			>
				<ExternalIcon aria-hidden="true" class="size-5" />
			</button>
		</div>
		<div class="h-px w-full bg-surface-5" />
		<div class="flex w-full flex-col gap-2.5">
			<div class="flex items-end gap-1 whitespace-nowrap">
				<span class="text-base font-semibold leading-5 text-contrast">
					{{ formattedLiveAmount }}
				</span>
				<span class="text-xs font-medium leading-4">of {{ formattedGoalAmount }}</span>
			</div>
			<ProgressBar
				class="pride-fundraiser-banner__progress"
				:progress="live_amount"
				:max="goal_amount"
				color="purple"
				full-width
				:gradient-border="false"
				:aria-label="progressLabel"
			/>
			<div class="flex flex-wrap items-center gap-2 text-xs font-medium leading-4">
				<span class="flex items-center gap-1">
					<UsersIcon aria-hidden="true" class="size-4 shrink-0" />
					6767 supporters
				</span>
				<span class="flex items-center gap-1">
					<CalendarIcon aria-hidden="true" class="size-4 shrink-0" />
					6 days left
				</span>
			</div>
		</div>
	</section>
</template>

<style scoped>
.pride-fundraiser-banner__progress :deep(.progress-bar) {
	background: linear-gradient(
		90deg,
		var(--color-red) 0%,
		var(--color-orange) 20%,
		var(--color-green) 50%,
		var(--color-blue) 75%,
		var(--color-purple) 100%
	);
}
</style>
