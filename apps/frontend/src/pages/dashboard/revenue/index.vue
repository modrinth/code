<template>
	<div class="flex flex-col gap-8 p-2">
		<div class="flex flex-col gap-5">
			<div class="flex flex-col">
				<span class="text-2xl text-contrast">Revenue</span>
				<span
					class="via-bg-brand-button bg-gradient-to-r from-brand-green via-20% to-brand-blue bg-clip-text text-4xl font-extrabold text-transparent"
					>{{ formatMoney(grandTotal) }}</span
				>
			</div>
			<div class="flex h-3 w-full overflow-hidden rounded-full">
				<span
					class="zone zone--green h-full flex-none bg-brand-green"
					:style="{ width: zoneGreenPct }"
				></span>
				<span
					class="zone zone--striped--green h-full flex-none bg-highlight"
					:style="{ width: zoneStripedPct }"
				></span>
				<span
					class="zone zone--striped--gray h-full flex-none bg-button-bg"
					:style="{ width: zoneBgPct }"
				></span>
			</div>
			<div class="grid grid-cols-3 gap-4">
				<div class="border-0 !border-r-[2px] border-solid border-button-bg">
					<span class="flex flex-row gap-2 align-middle text-xl"
						><span class="my-auto block size-4 rounded-full bg-brand-green px-2"></span> Available
						now</span
					>
					<span class="text-2xl font-bold text-contrast">{{ formatMoney(totalAvailable) }}</span>
				</div>
				<div class="border-0 !border-r-[2px] border-solid border-button-bg">
					<span class="flex flex-row gap-2 align-middle text-xl"
						><span
							class="zone--striped--green zone--striped-small my-auto block size-4 rounded-full bg-highlight px-2"
						></span>
						Available {{ nextDate?.date ? dayjs(nextDate.date).format('MMM D, YYYY') : '' }}</span
					>
					<span class="text-2xl font-bold text-contrast">{{
						formatMoney(nextDate?.amount ?? 0)
					}}</span>
				</div>
				<div>
					<div class="flex flex-row justify-between">
						<span class="flex flex-row gap-2 align-middle text-xl"
							><span
								class="zone--striped--gray zone--striped-small my-auto block size-4 rounded-full bg-button-bg px-2"
							></span>
							Processing</span
						>
						<span class="my-auto block size-5">
							<nuxt-link
								v-tooltip="`Click to read about how Modrinth handles your revenue.`"
								class="align-middle text-link"
								to="/legal/cmp-info#pending"
							>
								<UnknownIcon class="size-5" />
							</nuxt-link>
						</span>
					</div>
					<span class="text-2xl font-bold text-contrast">{{ formatMoney(remainingPending) }}</span>
				</div>
			</div>
		</div>
	</div>
</template>
<script setup lang="ts">
import { UnknownIcon } from '@modrinth/assets'
import { formatMoney } from '@modrinth/utils'
import dayjs from 'dayjs'

interface UserBalanceResponse {
	available: number
	pending: number
	// ISO 8601 date string: number
	dates: Record<string, number>
}

const { data: userBalance } = await useAsyncData(
	`payout/balance`,
	() => useBaseFetch(`payout/balance`, { apiVersion: 3 }) as Promise<UserBalanceResponse>,
)

const totalAvailable = computed(() => (userBalance.value ? Number(userBalance.value.available) : 0))
const nextDate = computed<{ date: string; amount: number } | null>(() => {
	const dates = userBalance.value?.dates
	if (!dates) return null

	const now = Date.now()

	const upcoming = Object.entries(dates)
		.map(([date, amount]) => ({ date, amount: Number(amount) }))
		.filter(({ date }) => new Date(date).getTime() > now)
		.sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime())[0]

	return upcoming ?? null
})
const remainingPending = computed(() => {
	return userBalance.value ? Number(userBalance.value.pending) - (nextDate.value?.amount ?? 0) : 0
})
const grandTotal = computed(
	() => totalAvailable.value + (nextDate.value?.amount ?? 0) + remainingPending.value,
)

const zoneGreen = ref(40)
const zoneStriped = ref(20)
const zoneGreenPct = computed(() => `${zoneGreen.value}%`)
const zoneStripedPct = computed(() => `${zoneStriped.value}%`)
const zoneBgPct = computed(() => `${Math.max(0, 100 - zoneGreen.value - zoneStriped.value)}%`)
</script>

<style scoped lang="scss">
.zone--striped--green,
.zone--striped--gray {
	background-attachment: fixed;
	background-position: 0 0;
	background-size: 9.38px 9.38px;
}

.zone--striped-small {
	background-size: 6.19px 6.19px !important;
	background-position: unset !important;
	background-attachment: unset !important;
}

.zone--striped--green {
	background-image: linear-gradient(
		135deg,
		var(--color-green) 11.54%,
		transparent 11.54%,
		transparent 50%,
		var(--color-green) 50%,
		var(--color-green) 61.54%,
		transparent 61.54%,
		transparent 100%
	);
}

.zone--striped--gray {
	background-image: linear-gradient(
		135deg,
		var(--color-button-bg-hover) 11.54%,
		transparent 11.54%,
		transparent 50%,
		var(--color-button-bg-hover) 50%,
		var(--color-button-bg-hover) 61.54%,
		transparent 61.54%,
		transparent 100%
	);
}
</style>
