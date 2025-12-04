<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
import {
	EditIcon,
	ExternalIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	RightArrowIcon,
	SignalIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { formatPrice, getPingLevel } from '@modrinth/utils'
import { useVIntl } from '@vintl/vintl'
import dayjs from 'dayjs'
import type Stripe from 'stripe'
import { computed } from 'vue'

import { getPriceForInterval, monthsInInterval } from '../../utils/product-utils'
import { regionOverrides } from '../../utils/regions'
import ButtonStyled from '../base/ButtonStyled.vue'
import Checkbox from '../base/Checkbox.vue'
import TagItem from '../base/TagItem.vue'
import ModrinthServersIcon from '../servers/ModrinthServersIcon.vue'
import ExpandableInvoiceTotal from './ExpandableInvoiceTotal.vue'
import FormattedPaymentMethod from './FormattedPaymentMethod.vue'
import type { ServerBillingInterval } from './ModrinthServersPurchaseModal.vue'
import ServersSpecs from './ServersSpecs.vue'

const vintl = useVIntl()
const { locale, formatMessage } = vintl

const emit = defineEmits<{
	(e: 'changePaymentMethod' | 'reloadPaymentIntent'): void
}>()

const props = defineProps<{
	plan: Labrinth.Billing.Internal.Product
	region: Archon.Servers.v1.Region
	tax?: number
	total?: number
	currency: string
	ping?: number
	loading?: boolean
	selectedPaymentMethod: Stripe.PaymentMethod | undefined
	hasPaymentMethod?: boolean
	noPaymentRequired?: boolean
	existingPlan?: Labrinth.Billing.Internal.Product
	existingSubscription?: Labrinth.Billing.Internal.UserSubscription
}>()

const interval = defineModel<ServerBillingInterval>('interval', { required: true })
const acceptedEula = defineModel<boolean>('acceptedEula', { required: true })

const selectedPlanPriceForInterval = computed<number | undefined>(() => {
	return getPriceForInterval(props.plan, props.currency, interval.value)
})

const existingPlanPriceForInterval = computed<number | undefined>(() => {
	if (!props.existingPlan) return undefined
	return getPriceForInterval(props.existingPlan, props.currency, interval.value)
})

const monthlyPrice = computed<number | undefined>(() => {
	return getPriceForInterval(props.plan, props.currency, 'monthly')
})

const quarterlyPrice = computed<number | undefined>(() => {
	return getPriceForInterval(props.plan, props.currency, 'quarterly')
})

const upgradeDeltaPrice = computed<number | undefined>(() => {
	if (selectedPlanPriceForInterval.value == null || existingPlanPriceForInterval.value == null)
		return undefined
	return selectedPlanPriceForInterval.value - existingPlanPriceForInterval.value
})

const isUpgrade = computed<boolean>(() => {
	return (upgradeDeltaPrice.value ?? 0) > 0
})

const estimatedDaysInInterval = computed<number>(() => {
	return monthsInInterval[interval.value] * 30
})

const estimatedProrationDays = computed<number | undefined>(() => {
	if (!isUpgrade.value) return undefined
	if (props.total == null || props.tax == null) return undefined
	const subtotal = props.total - props.tax
	const delta = upgradeDeltaPrice.value ?? 0
	if (delta <= 0) return undefined
	const fraction = Math.max(0, Math.min(1, subtotal / delta))
	return Math.round(fraction * estimatedDaysInInterval.value)
})

const isProratedCharge = computed<boolean>(() => {
	return isUpgrade.value && (props.total ?? 0) > 0
})

const exactProrationDays = computed<number | undefined>(() => {
	if (!props.existingSubscription) return undefined
	const created = dayjs(props.existingSubscription.created)
	if (!created.isValid()) return undefined
	let next = created
	const now = dayjs()
	if (props.existingSubscription.interval === 'monthly') {
		const cycles = now.diff(created, 'month')
		next = created.add(cycles + 1, 'month')
	} else if (props.existingSubscription.interval === 'quarterly') {
		const months = now.diff(created, 'month')
		const cycles = Math.floor(months / 3)
		next = created.add((cycles + 1) * 3, 'month')
	} else if (props.existingSubscription.interval === 'yearly') {
		const cycles = now.diff(created, 'year')
		next = created.add(cycles + 1, 'year')
	} else if (props.existingSubscription.interval === 'five-days') {
		const days = now.diff(created, 'day')
		const cycles = Math.floor(days / 5)
		next = created.add((cycles + 1) * 5, 'day')
	} else {
		return undefined
	}
	const days = next.diff(now, 'day')
	return Math.max(0, days)
})

const prorationDays = computed<number | undefined>(
	() => exactProrationDays.value ?? estimatedProrationDays.value,
)

const planName = computed(() => {
	if (!props.plan || !props.plan.metadata || props.plan.metadata.type !== 'pyro') return 'Unknown'
	const ram = props.plan.metadata.ram
	if (ram === 4096) return 'Small'
	if (ram === 6144) return 'Medium'
	if (ram === 8192) return 'Large'
	return 'Custom'
})

const planSpecs = computed(() => {
	const metadata = props.plan.metadata
	if (metadata.type === 'pyro' || metadata.type === 'medal') {
		return {
			ram: metadata.ram,
			storage: metadata.storage,
			cpu: metadata.cpu,
		}
	}
	return null
})

const flag = computed(
	() =>
		regionOverrides[props.region.shortcode]?.flag ??
		`https://flagcdn.com/${props.region.country_code}.svg`,
)
const overrideTitle = computed(() => regionOverrides[props.region.shortcode]?.name)
const title = computed(() =>
	overrideTitle.value ? formatMessage(overrideTitle.value) : props.region.display_name,
)
const locationSubtitle = computed(() =>
	overrideTitle.value ? props.region.display_name : undefined,
)
const pingLevel = computed(() => getPingLevel(props.ping ?? 0))

const period = computed(() => {
	if (interval.value === 'monthly') return 'month'
	if (interval.value === 'quarterly') return '3 months'
	if (interval.value === 'yearly') return 'year'
	return '???'
})

function setInterval(newInterval: ServerBillingInterval) {
	interval.value = newInterval
	emit('reloadPaymentIntent')
}
</script>

<template>
	<div class="grid sm:grid-cols-[3fr_2fr] gap-4">
		<div class="bg-table-alternateRow p-4 rounded-2xl">
			<div class="flex items-center gap-2 mb-3">
				<ModrinthServersIcon class="flex h-5 w-fit" />
				<TagItem>{{ planName }}</TagItem>
			</div>
			<div>
				<ServersSpecs
					v-if="planSpecs"
					class="!grid sm:grid-cols-2"
					:ram="planSpecs.ram"
					:storage="planSpecs.storage"
					:cpus="planSpecs.cpu"
				/>
			</div>
		</div>
		<div
			class="bg-table-alternateRow p-4 rounded-2xl flex flex-col gap-2 items-center justify-center"
		>
			<img
				v-if="flag"
				class="aspect-[16/10] max-w-12 w-full object-cover rounded-md border-1 border-button-border border-solid"
				:src="flag"
				alt=""
				aria-hidden="true"
			/>
			<span class="font-semibold">
				{{ title }}
			</span>
			<span class="text-xs flex items-center gap-1 text-secondary font-medium">
				<template v-if="locationSubtitle">
					<span>
						{{ locationSubtitle }}
					</span>
					<span v-if="ping !== -1">•</span>
				</template>
				<template v-if="ping !== -1">
					<SignalIcon
						v-if="ping"
						aria-hidden="true"
						:style="`--_signal-${pingLevel}: ${pingLevel <= 2 ? 'var(--color-red)' : pingLevel <= 4 ? 'var(--color-orange)' : 'var(--color-green)'}`"
						stroke-width="3px"
						class="shrink-0"
					/>
					<SpinnerIcon v-else class="animate-spin" />
					<template v-if="ping"> {{ ping }}ms </template>
					<span v-else> Testing connection... </span>
				</template>
			</span>
		</div>
	</div>

	<div class="grid grid-cols-2 gap-2 mt-4">
		<button
			:class="
				interval === 'monthly'
					? 'bg-button-bg border-transparent'
					: 'bg-transparent  border-button-border'
			"
			class="rounded-2xl active:scale-[0.98] transition-transform duration-100 border-2 border-solid p-4 flex items-center gap-2"
			@click="setInterval('monthly')"
		>
			<RadioButtonCheckedIcon v-if="interval === 'monthly'" class="size-6 text-brand" />
			<RadioButtonIcon v-else class="size-6 text-secondary" />
			<div class="flex flex-col items-start gap-1 font-medium text-primary">
				<span class="flex items-center gap-1" :class="{ 'text-contrast': interval === 'monthly' }"
					>Pay monthly</span
				>
				<span class="text-sm text-secondary flex items-center gap-1"
					>{{ formatPrice(locale, monthlyPrice, currency, true) }} / month</span
				>
			</div>
		</button>
		<button
			:class="
				interval === 'quarterly'
					? 'bg-button-bg border-transparent'
					: 'bg-transparent  border-button-border'
			"
			class="rounded-2xl active:scale-[0.98] transition-transform duration-100 border-2 border-solid p-4 flex items-center gap-2"
			@click="setInterval('quarterly')"
		>
			<RadioButtonCheckedIcon v-if="interval === 'quarterly'" class="size-6 text-brand" />
			<RadioButtonIcon v-else class="size-6 text-secondary" />
			<div class="flex flex-col items-start gap-1 font-medium text-primary">
				<span class="flex items-center gap-1" :class="{ 'text-contrast': interval === 'quarterly' }"
					>Pay quarterly
					<span class="text-xs font-bold text-brand px-1.5 py-0.5 rounded-full bg-brand-highlight"
						>{{ interval === 'quarterly' ? 'Saving' : 'Save' }} 16%</span
					></span
				>
				<span class="text-sm text-secondary flex items-center gap-1"
					>{{
						formatPrice(
							locale,
							(quarterlyPrice ?? 0) / monthsInInterval['quarterly'],
							currency,
							true,
						)
					}}
					/ month</span
				>
			</div>
		</button>
	</div>
	<div class="mt-2">
		<template v-if="!noPaymentRequired">
			<ExpandableInvoiceTotal
				:period="isProratedCharge ? undefined : period"
				:currency="currency"
				:loading="loading"
				:total="total ?? -1"
				:billing-items="
					total !== undefined && tax !== undefined
						? [
								{
									title:
										isProratedCharge && prorationDays
											? `Modrinth Hosting (${planName}) — prorated for ${prorationDays} day${
													prorationDays === 1 ? '' : 's'
												}`
											: `Modrinth Hosting (${planName})`,
									amount: total - tax,
								},
								{
									title: 'Tax',
									amount: tax,
								},
							]
						: []
				"
			/>
		</template>
		<div
			v-else
			class="p-4 rounded-2xl bg-table-alternateRow text-sm text-secondary leading-relaxed"
		>
			No payment required. Your downgrade will apply at the end of the current billing period.
		</div>
	</div>
	<div
		v-if="!noPaymentRequired"
		class="mt-2 flex items-center pl-4 pr-2 py-3 bg-bg rounded-2xl gap-2 text-secondary"
	>
		<template v-if="selectedPaymentMethod">
			<FormattedPaymentMethod :method="selectedPaymentMethod" />
		</template>
		<template v-else>
			<div v-if="hasPaymentMethod" class="flex items-center gap-2 text-secondary">
				<RadioButtonCheckedIcon class="text-brand" />
				Using new payment method
			</div>
			<div v-else class="flex items-center gap-2 text-red">
				<XIcon />
				No payment method selected
			</div>
		</template>
		<ButtonStyled size="small" type="transparent">
			<button class="ml-auto" @click="emit('changePaymentMethod')">
				<template v-if="selectedPaymentMethod || hasPaymentMethod"> <EditIcon /> Change </template>
				<template v-else> Select payment method <RightArrowIcon /> </template>
			</button>
		</ButtonStyled>
	</div>
	<p v-if="!noPaymentRequired" class="m-0 mt-4 text-sm text-secondary">
		<template v-if="isUpgrade && (total ?? 0) > 0">
			Today, you will be charged a prorated amount for the remainder of your current billing cycle.
			<br />
			Your subscription will renew at
			{{ formatPrice(locale, selectedPlanPriceForInterval, currency) }} / {{ period }} plus
			applicable taxes at the end of your current billing interval, until you cancel. You can cancel
			anytime from your settings page.
		</template>
		<template v-else>
			You'll be charged
			<SpinnerIcon v-if="loading" class="animate-spin relative top-0.5 mx-2" /><template v-else>{{
				formatPrice(locale, total, currency)
			}}</template>
			every {{ period }} plus applicable taxes starting today, until you cancel. You can cancel
			anytime from your settings page.
		</template>
		<br />
		<span class="font-semibold"
			>By clicking "Subscribe", you are purchasing a recurring subscription.</span
		>
		<br />
	</p>
	<div v-if="!noPaymentRequired" class="mt-2 flex items-center gap-1 text-sm">
		<Checkbox
			v-model="acceptedEula"
			label="I acknowledge that I have read and agree to the"
			description="I acknowledge that I have read and agree to the Minecraft EULA"
		/>
		<a
			href="https://www.minecraft.net/en-us/eula"
			target="_blank"
			class="text-brand underline hover:brightness-[--hover-brightness]"
			>Minecraft EULA<ExternalIcon class="size-3 shrink-0 ml-0.5 mb-0.5"
		/></a>
	</div>
</template>
