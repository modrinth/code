<script setup lang="ts">
import { CalendarIcon, UsersIcon, XIcon } from '@modrinth/assets'
import { injectModrinthClient, ProgressBar } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, ref } from 'vue'

const DISMISSED_STORAGE_KEY = 'pride-fundraiser-2026-dismissed'

const client = injectModrinthClient()
const dismissed = ref(localStorage.getItem(DISMISSED_STORAGE_KEY) === 'true')

const { data: campaignInfo } = useQuery({
	queryKey: ['campaign', 'pride-26'],
	queryFn: () => client.labrinth.campaign_internal.getPride26(),
	enabled: () => !dismissed.value,
	staleTime: 15 * 60 * 1000,
	refetchInterval: 15 * 60 * 1000,
	retry: false,
})
const shouldShowBanner = computed(
	() => !dismissed.value && Number(campaignInfo.value?.target_usd) > 0,
)

async function openPrideFundraiser() {
	await openUrl('https://modrinth.com/pride?from=app')
}

function dismissBanner() {
	dismissed.value = true
	localStorage.setItem(DISMISSED_STORAGE_KEY, 'true')
}

function formatUsd(amount: string | number) {
	return Number(amount).toLocaleString('en-US', {
		style: 'currency',
		currency: 'USD',
		maximumFractionDigits: 0,
	})
}

function daysLeft() {
	return Math.max(
		0,
		Math.ceil((new Date('2026-07-01T00:00:00Z').getTime() - Date.now()) / (24 * 60 * 60 * 1000)),
	)
}
</script>

<template>
	<div v-if="shouldShowBanner && campaignInfo">
		<section
			role="link"
			tabindex="0"
			class="flex w-full cursor-pointer flex-col gap-3 rounded-xl border border-solid border-surface-5 bg-button-bg p-3 text-primary transition-[border-color,filter] hover:border-surface-6 hover:brightness-125 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
			aria-label="Open Pride fundraiser"
			@click="openPrideFundraiser"
			@keydown.enter="openPrideFundraiser"
			@keydown.space.prevent="openPrideFundraiser"
		>
			<div class="flex w-full items-center justify-between gap-2">
				<h2 class="m-0 min-w-0 truncate text-base font-semibold text-contrast">
					Pride Fundraiser 2026
				</h2>
				<button
					type="button"
					class="m-0 flex size-5 shrink-0 cursor-pointer items-center justify-center border-0 bg-transparent p-0 text-primary transition-colors hover:text-contrast focus-visible:text-contrast"
					aria-label="Dismiss Pride fundraiser"
					@click.stop="dismissBanner"
					@keydown.stop
				>
					<XIcon aria-hidden="true" class="size-5" />
				</button>
			</div>
			<div class="h-px w-full bg-surface-5" />
			<div class="flex w-full flex-col gap-2.5">
				<div class="flex items-end gap-1 whitespace-nowrap">
					<span class="text-base font-semibold leading-5 text-contrast">
						{{ formatUsd(campaignInfo.total_donations_usd) }}
					</span>
					<span class="text-xs font-medium leading-4">
						of {{ formatUsd(campaignInfo.target_usd) }}
					</span>
				</div>
				<ProgressBar
					class="pride-fundraiser-banner__progress"
					:progress="Number(campaignInfo.total_donations_usd)"
					:max="Number(campaignInfo.target_usd)"
					color="purple"
					full-width
					:gradient-border="false"
					:aria-label="`${formatUsd(campaignInfo.total_donations_usd)} of ${formatUsd(
						campaignInfo.target_usd,
					)} raised`"
				/>
				<div class="flex flex-wrap items-center gap-2 text-xs font-medium leading-4">
					<span class="flex items-center gap-1">
						<UsersIcon aria-hidden="true" class="size-4 shrink-0" />
						{{ campaignInfo.num_donators.toLocaleString('en-US') }}
						{{ campaignInfo.num_donators === 1 ? 'supporter' : 'supporters' }}
					</span>
					<span class="flex items-center gap-1">
						<CalendarIcon aria-hidden="true" class="size-4 shrink-0" />
						{{ daysLeft() }} {{ daysLeft() === 1 ? 'day left' : 'days left' }}
					</span>
				</div>
			</div>
		</section>
	</div>
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
