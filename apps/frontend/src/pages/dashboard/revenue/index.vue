<template>
	<CreatorWithdrawModal
		ref="withdrawModal"
		:balance="userBalance"
		v-model:country="withdrawCountry"
		:payout-methods="paymentMethods"
		:payout-methods-pending="payoutMethodsPending"
	/>
	<div class="mb-6 flex flex-col gap-8 p-2">
		<div class="flex flex-col gap-5">
			<div class="flex flex-col">
				<span class="text-2xl font-semibold text-contrast">Balance</span>
				<span
					class="bg-gradient-to-r from-brand-purple via-brand-orange via-20% to-brand-orange bg-clip-text text-4xl font-extrabold text-transparent"
					>{{ formatMoney(grandTotal) }}</span
				>
			</div>
			<div class="flex h-3 w-full gap-2 overflow-hidden rounded-full bg-bg-raised">
				<template v-for="seg in segments" :key="seg.key">
					<span
						v-tooltip="formatMoney(seg.amount)"
						class="block h-full"
						:class="seg.class"
						:style="{ width: seg.widthPct }"
					></span>
				</template>
			</div>
			<div class="flex flex-col">
				<div
					class="flex flex-row justify-between border-0 !border-b-[2px] border-solid border-button-bg p-2"
				>
					<span class="text-md flex flex-row gap-2 align-middle"
						><span class="my-auto block size-4 rounded-full bg-brand-green"></span> Available
						now</span
					>
					<span class="text-md font-bold text-contrast">{{ formatMoney(totalAvailable) }}</span>
				</div>
				<div
					class="flex flex-row justify-between border-0 !border-b-[2px] border-solid border-button-bg p-2"
					v-for="date in dateSegments"
					:key="date.date"
				>
					<span class="text-md flex flex-row gap-2 align-middle"
						><span
							class="zone--striped-small my-auto block size-4 rounded-full"
							:class="[date.stripeClass, date.highlightClass]"
						></span>
						Estimated {{ date.date ? dayjs(date.date).format('MMM D, YYYY') : '' }}
						<Tooltip :triggers="['hover', 'focus']">
							<nuxt-link class="align-middle text-link" to="/legal/cmp-info#pending">
								<UnknownIcon />
							</nuxt-link>
							<template #popper>
								<div class="w-[250px] font-semibold text-contrast">
									Estimated revenue may be subject to change until it is made available.<br /><br />Click
									to read about how Modrinth handles your revenue.
								</div>
							</template>
						</Tooltip>
					</span>
					<span class="text-md font-bold text-contrast">{{ formatMoney(date?.amount ?? 0) }}</span>
				</div>
				<div
					class="flex flex-row justify-between border-0 !border-b-[2px] border-solid border-button-bg p-2"
				>
					<span class="text-md flex flex-row gap-2 align-middle"
						><span
							class="zone--striped-small zone--striped--gray my-auto block size-4 rounded-full bg-button-bg"
						></span>
						Processing
						<Tooltip :triggers="['hover', 'focus']">
							<InProgressIcon class="my-auto" />
							<template #popper>
								<div class="w-[250px] font-semibold text-contrast">
									Revenue stays in processing until the end of the month, then becomes available 60
									days later.
								</div>
							</template>
						</Tooltip>
					</span>
					<span class="text-md font-bold text-contrast">{{
						formatMoney(processingDate?.amount ?? 0)
					}}</span>
				</div>
			</div>
		</div>
		<div class="flex flex-col gap-4">
			<span class="text-3xl font-semibold text-contrast">Withdraw</span>
			<div class="grid grid-cols-3 gap-x-4 gap-y-2">
				<button
					@click="openWithdrawModal"
					class="flex flex-col rounded-2xl bg-brand p-5 text-inverted shadow-xl brightness-90 transition-all duration-200 hover:brightness-105"
				>
					<div class="flex flex-row justify-between">
						<span class="text-lg font-semibold">Withdraw</span>
						<ArrowUpRightIcon class="my-auto size-4" />
					</div>
					<div class="text-left">Withdraw from your available balance to any payout method.</div>
				</button>
				<button
					class="flex flex-col rounded-2xl bg-button-bg p-5 shadow-xl brightness-90 transition-all duration-200 hover:brightness-105"
				>
					<div class="flex flex-row justify-between">
						<span class="text-lg font-semibold text-contrast">PayPal</span>
						<ArrowUpRightIcon class="my-auto size-4" />
					</div>
					<div class="text-left text-primary">
						Withdraw from your available balance to PayPal again.
					</div>
				</button>
			</div>
			<span class="text-sm text-secondary"
				>By uploading projects to Modrinth and withdrawing money from your account, you agree to the
				<nuxt-link class="text-link" to="/legal/cmp">Rewards Program Terms</nuxt-link>. Learn more
				about the
				<nuxt-link class="text-link" to="/legal/cmp-info">Reward Program</nuxt-link>.</span
			>
		</div>
		<div class="flex flex-col gap-4" v-if="sortedPayouts.length > 0">
			<div class="flex flex-row justify-between">
				<span class="text-3xl font-semibold text-contrast">Transactions</span>
				<nuxt-link
					class="my-auto font-semibold text-contrast underline underline-offset-2"
					to="/dashboard/revenue/transfers"
					>See all</nuxt-link
				>
			</div>
			<div v-for="transaction in sortedPayouts.slice(0, 3)" class="flex flex-row gap-3">
				<div
					class="flex size-12 justify-center rounded-full border-[1px] border-solid border-button-bg bg-bg-raised shadow-md"
				>
					<ArrowUpIcon class="my-auto size-8 text-contrast" />
				</div>
				<div class="flex w-full flex-row justify-between">
					<div class="flex flex-col">
						<span class="text-lg font-semibold text-contrast">{{
							formatMethodName(transaction.method)
						}}</span>
						<span class="text-secondary"
							>{{ formatTransactionStatus(transaction.status) }} |
							{{ dayjs(transaction.created).format('MMM DD YYYY') }}</span
						>
					</div>
					<span class="my-auto text-2xl font-bold text-contrast">{{
						formatMoney(transaction.amount)
					}}</span>
				</div>
			</div>
		</div>
	</div>
</template>
<script setup lang="ts">
import { ArrowUpIcon, ArrowUpRightIcon, InProgressIcon, UnknownIcon } from '@modrinth/assets'
import {
	capitalizeString,
	formatMoney,
	type PayoutList,
	type PayoutMethodType,
	type PayoutStatus,
} from '@modrinth/utils'
import dayjs from 'dayjs'
import { Tooltip } from 'floating-vue'
import { all } from 'iso-3166-1'
import CreatorWithdrawModal from '~/components/ui/dashboard/CreatorWithdrawModal.vue'

// TODO: Deduplicate in @modrinth/api-client PR.
type FormCompletionStatus = 'unknown' | 'unrequested' | 'unsigned' | 'tin-mismatch' | 'complete'

interface UserBalanceResponse {
	available: number
	withdrawn_lifetime: number
	withdrawn_ytd: number
	pending: number
	// ISO 8601 date string -> amount
	dates: Record<string, number>
	// backend returns null when not applicable
	requested_form_type: string | null
	form_completion_status: FormCompletionStatus | null
}

// Types for payout methods and related shapes
type PayoutInterval = { fixed: { values: number[] } } | { standard: { min: number; max: number } }
interface PayoutMethodFee {
	percentage: number
	min: number
	max?: number | null
}
interface PayoutMethod {
	id: string
	name: string
	type: PayoutMethodType
	supported_countries: string[]
	image_url?: string | null
	interval: PayoutInterval
	fee: PayoutMethodFee
}

const countries = computed(() =>
	all().map((x) => ({
		id: x.alpha2,
		name: x.alpha2 === 'TW' ? 'Taiwan' : x.country,
	})),
)
const withdrawCountry = ref<{ id: string; name: string } | null>(null)

if (!withdrawCountry.value) {
	const us = countries.value.find((c) => c.id === 'US')
	withdrawCountry.value = us ?? countries.value[0] ?? null
}

type RevenueBarSegment = {
	key: string
	class: string
	widthPct: string
	amount: number
}

const withdrawModal = ref<InstanceType<typeof CreatorWithdrawModal>>()
async function openWithdrawModal() {
	withdrawModal.value?.show?.()
	console.log(withdrawModal.value?.show)
}

function formatTransactionStatus(status: PayoutStatus) {
	switch (status) {
		case 'in-transit':
			return 'In Transit'
		default:
			return capitalizeString(status)
	}
}

function formatMethodName(method: PayoutMethodType | null) {
	if (!method) return 'Unknown'
	switch (method) {
		case 'paypal':
			return 'PayPal'
		case 'venmo':
			return 'Venmo'
		case 'tremendous':
			return 'Tremendous'
		default:
			return capitalizeString(method)
	}
}

const flags = useFeatureFlags()

const { data: userBalance } = await useAsyncData(
	`payout/balance`,
	() => useBaseFetch(`payout/balance`, { apiVersion: 3 }) as Promise<UserBalanceResponse>,
)

const { data: payouts } = await useAsyncData<PayoutList>(
	`payout`,
	() =>
		useBaseFetch(`payout`, {
			apiVersion: 3,
		}) as Promise<PayoutList>,
)

const sortedPayouts = computed<PayoutList>(() => {
	if (!payouts.value) return []

	return payouts.value.sort((a, b) => {
		return new Date(b.created).getTime() - new Date(a.created).getTime()
	})
})

// Fetch payout methods based on selected country
const { data: payoutMethods, pending: payoutMethodsPending } = await useAsyncData(
	'payout-methods',
	() =>
		useBaseFetch(`payout/methods?country=${withdrawCountry.value?.id ?? 'US'}`, { apiVersion: 3 }),
	{ default: () => [] as PayoutMethod[], watch: [withdrawCountry] },
)

const paymentMethods = computed<PayoutMethod[]>(() => (payoutMethods.value as PayoutMethod[]) ?? [])

const totalAvailable = computed(() => (userBalance.value ? Number(userBalance.value.available) : 0))
const nextDate = computed<{ date: string; amount: number }[]>(() => {
	const dates = userBalance.value?.dates
	if (!dates) return []

	const now = Date.now()

	return Object.entries(dates)
		.map(([date, amount]) => ({ date, amount: Number(amount) }))
		.filter(({ date }) => new Date(date).getTime() > now)
		.sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime())
})

const processingDate = computed<{ date: string; amount: number }>(() => {
	const nextDates = nextDate.value
	if (!nextDates.length) return { date: '', amount: 0 }

	return nextDates[nextDates.length - 1]
})

const grandTotal = computed(() =>
	userBalance.value ? Number(userBalance.value.available) + Number(userBalance.value.pending) : 0,
)

const dateStripeClasses = [
	'zone--striped--blue',
	'zone--striped--purple',
	'zone--striped--orange',
	'zone--striped--red',
] as const

const dateHighlightClasses = [
	'bg-highlight-blue',
	'bg-highlight-purple',
	'bg-highlight-orange',
	'bg-highlight-red',
] as const

const dateSegments = computed(() => {
	const dates = nextDate.value
	if (!dates?.length)
		return [] as Array<{
			date: string
			amount: number
			stripeClass: string
			highlightClass: string
		}>

	return dates.slice(0, -1).map((d, i) => ({
		...d,
		stripeClass: dateStripeClasses[i % dateStripeClasses.length],
		highlightClass: dateHighlightClasses[i % dateHighlightClasses.length],
	}))
})

const segments = computed<RevenueBarSegment[]>(() => {
	const available = totalAvailable.value || 0
	const dates = nextDate.value || []
	const processing = processingDate.value

	const upcoming = dates.slice(0, Math.max(0, dates.length - 1))
	const totalPending = dates.reduce((sum, d) => sum + (Number(d.amount) || 0), 0)
	const total = available + totalPending

	if (total <= 0) return [] as RevenueBarSegment[]

	const segs: Array<{ key: string; class: string; width: number; amount: number }> = []

	if (available > 0) {
		segs.push({
			key: 'available',
			class: 'bg-brand-green',
			width: available / total,
			amount: available,
		})
	}

	upcoming.forEach((d, i) => {
		const amt = Number(d.amount) || 0
		if (amt <= 0) return
		const stripe = dateStripeClasses[i % dateStripeClasses.length]
		const hi = dateHighlightClasses[i % dateHighlightClasses.length]
		segs.push({
			key: `upcoming-${d.date}-${i}`,
			class: `${stripe} ${hi}`,
			width: amt / total,
			amount: amt,
		})
	})

	if (processing?.amount) {
		segs.push({
			key: 'processing',
			class: 'zone--striped--gray bg-button-bg',
			width: (Number(processing.amount) || 0) / total,
			amount: Number(processing.amount) || 0,
		})
	}

	let acc = 0
	// normalize widths to sum to 100%, then drop any that are 0% and re-adjust the last
	const normalized = segs.map((s, idx) => {
		let pct = Math.round(s.width * 10000) / 100 // keep 2 decimals
		if (idx === segs.length - 1) {
			pct = Math.max(0, 100 - acc)
		}
		acc += pct
		return { key: s.key, class: s.class, pct, amount: s.amount }
	})

	const filtered = normalized.filter((s) => s.pct > 0)
	if (!filtered.length) return [] as RevenueBarSegment[]

	const sumExceptLast = filtered.slice(0, -1).reduce((sum, s) => sum + s.pct, 0)
	filtered[filtered.length - 1].pct = Math.max(0, 100 - sumExceptLast)

	return filtered.map((s) => ({
		key: s.key,
		class: s.class,
		widthPct: `${s.pct}%`,
		amount: s.amount,
	})) as RevenueBarSegment[]
})
</script>

<style scoped lang="scss">
%zone--striped-common {
	background-attachment: fixed;
	background-position: 0 0;
	background-size: 9.38px 9.38px;
}

@mixin striped-background($color-variable) {
	background-image: linear-gradient(
		135deg,
		$color-variable 11.54%,
		transparent 11.54%,
		transparent 50%,
		$color-variable 50%,
		$color-variable 61.54%,
		transparent 61.54%,
		transparent 100%
	);
}

$striped-colors: 'green', 'blue', 'purple', 'orange', 'red';
@each $color in $striped-colors {
	.zone--striped--#{$color} {
		@include striped-background(var(--color-#{$color}));
		@extend %zone--striped-common;
	}
}

.zone--striped--gray {
	@extend %zone--striped-common;
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

.zone--striped-small {
	background-size: 6.19px 6.19px !important;
	background-position: unset !important;
	background-attachment: unset !important;
}
</style>
