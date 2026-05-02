<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { RightArrowIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { useFormatPrice } from '../../composables'
import { defineMessages, useVIntl } from '../../composables/i18n'
import { getPriceForInterval, monthsInInterval } from '../../utils/product-utils'
import ButtonStyled from '../base/ButtonStyled.vue'
import OptionGroup from '../base/OptionGroup.vue'
import type { ServerBillingInterval } from './ModrinthServersPurchaseModal.vue'
import ServersSpecs from './ServersSpecs.vue'

const { formatMessage } = useVIntl()
const formatPrice = useFormatPrice()

const props = defineProps<{
	availableProducts: Labrinth.Billing.Internal.Product[]
	currency: string
	existingPlan?: Labrinth.Billing.Internal.Product
}>()

const availableBillingIntervals = ['monthly', 'quarterly']

const selectedPlan = defineModel<Labrinth.Billing.Internal.Product>('plan')
const selectedInterval = defineModel<ServerBillingInterval>('interval')
const emit = defineEmits<{
	(e: 'choose-custom' | 'proceed'): void
}>()

const messages = defineMessages({
	selectPlan: {
		id: 'servers.purchase.step.plan.select',
		defaultMessage: 'Select Plan',
	},
	getStarted: {
		id: 'servers.purchase.step.plan.get-started',
		defaultMessage: 'Get started',
	},
	smallTitle: {
		id: 'servers.purchase.step.plan.small',
		defaultMessage: 'Small',
	},
	mediumTitle: {
		id: 'servers.purchase.step.plan.medium',
		defaultMessage: 'Medium',
	},
	largeTitle: {
		id: 'servers.purchase.step.plan.large',
		defaultMessage: 'Large',
	},
	smallDesc: {
		id: 'servers.purchase.step.plan.small.desc',
		defaultMessage: 'Perfect for 1–5 friends with a few light mods.',
	},
	mediumDesc: {
		id: 'servers.purchase.step.plan.medium.desc',
		defaultMessage: 'Great for 6–15 players and multiple mods.',
	},
	largeDesc: {
		id: 'servers.purchase.step.plan.large.desc',
		defaultMessage: 'Ideal for 15–25 players, modpacks, or heavy modding.',
	},
	customDesc: {
		id: 'servers.purchase.step.plan.custom.desc',
		defaultMessage: 'Pick a customized plan with just the specs you need.',
	},
	mostPopular: {
		id: 'servers.purchase.step.plan.most-popular',
		defaultMessage: 'Most Popular',
	},
	billingSubtitle: {
		id: 'servers.purchase.step.plan.billing-subtitle',
		defaultMessage: 'Available in North America, Europe, and Southeast Asia.',
	},
	customHeading: {
		id: 'servers.purchase.step.plan.custom.heading',
		defaultMessage: 'Know exactly what you need?',
	},
	yourCurrentPlan: {
		id: 'servers.purchase.step.plan.your-current-plan',
		defaultMessage: 'Your current plan',
	},
})

const isSameAsExistingPlan = computed(() => {
	return !!(
		props.existingPlan &&
		selectedPlan.value &&
		props.existingPlan.id === selectedPlan.value.id
	)
})

const plansByRam = computed(() => {
	const byName: Record<
		'small' | 'medium' | 'large',
		Labrinth.Billing.Internal.Product | undefined
	> = {
		small: undefined,
		medium: undefined,
		large: undefined,
	}
	for (const p of props.availableProducts) {
		if (p?.metadata?.type !== 'pyro') continue
		if (p.metadata.ram === 4096) byName.small = p
		else if (p.metadata.ram === 6144) byName.medium = p
		else if (p.metadata.ram === 8192) byName.large = p
	}
	return byName
})

function planSpecs(plan: Labrinth.Billing.Internal.Product) {
	const m = plan.metadata
	if (m.type === 'pyro' || m.type === 'medal') {
		return { ram: m.ram, storage: m.storage, cpus: m.cpu }
	}
	return null
}

function pricePerMonth(plan?: Labrinth.Billing.Internal.Product) {
	if (!plan || !selectedInterval.value) return undefined
	const total = getPriceForInterval(plan, props.currency, selectedInterval.value)
	if (!total) return undefined
	return total / monthsInInterval[selectedInterval.value]
}

const customStartingPrice = computed(() => {
	let min: number | undefined
	for (const p of props.availableProducts) {
		const perMonth = pricePerMonth(p)
		if (perMonth && (min === undefined || perMonth < min)) min = perMonth
	}
	return min
})

const smallPrice = computed(() => pricePerMonth(plansByRam.value.small))
const mediumPrice = computed(() => pricePerMonth(plansByRam.value.medium))
const largePrice = computed(() => pricePerMonth(plansByRam.value.large))

const smallSpecs = computed(() =>
	plansByRam.value.small ? planSpecs(plansByRam.value.small) : null,
)
const mediumSpecs = computed(() =>
	plansByRam.value.medium ? planSpecs(plansByRam.value.medium) : null,
)
const largeSpecs = computed(() =>
	plansByRam.value.large ? planSpecs(plansByRam.value.large) : null,
)

function selectPlan(plan: Labrinth.Billing.Internal.Product) {
	selectedPlan.value = plan
	emit('proceed')
}

function selectCustom() {
	emit('choose-custom')
	emit('proceed')
}
</script>

<template>
	<div class="flex flex-col items-center gap-2 mb-5 !mt-0">
		<OptionGroup
			v-slot="{ option }"
			v-model="selectedInterval"
			class="!bg-button-bg !shadow-none"
			:options="availableBillingIntervals"
		>
			<template v-if="option === 'monthly'">Monthly</template>
			<span v-else-if="option === 'quarterly'">
				Quarterly <span class="text-brand">(Save 16%)</span>
			</span>
		</OptionGroup>
		<div class="text-sm text-secondary text-center">
			{{ formatMessage(messages.billingSubtitle) }}
		</div>
	</div>
	<Transition
		enter-active-class="transition-all duration-300 ease-out"
		enter-from-class="opacity-0 max-h-0"
		enter-to-class="opacity-100 max-h-20"
		leave-active-class="transition-all duration-200 ease-in"
		leave-from-class="opacity-100 max-h-20"
		leave-to-class="opacity-0 max-h-0"
	>
		<div v-if="isSameAsExistingPlan" class="text-orange mb-5 text-center" role="alert">
			Your server is already on this plan, choose a different plan.
		</div>
	</Transition>
	<div class="grid grid-cols-3 gap-4 items-start">
		<!-- Small -->
		<div
			v-if="plansByRam.small && smallPrice"
			class="flex flex-col gap-4 rounded-2xl bg-surface-2 border-2 border-solid border-transparent p-5 h-full"
		>
			<div>
				<div class="text-3xl font-semibold text-contrast leading-none">
					{{ formatMessage(messages.smallTitle) }}
				</div>
				<div class="mt-1">
					<span class="text-2xl font-bold text-contrast">
						{{ formatPrice(smallPrice, currency, true) }}
					</span>
					<span class="text-sm">
						/ month<template v-if="selectedInterval !== 'monthly'"
							>, billed {{ selectedInterval }}</template
						>
					</span>
				</div>
				<div class="mt-2 text-sm text-primary">
					{{ formatMessage(messages.smallDesc) }}
				</div>
			</div>
			<div class="w-full">
				<ButtonStyled color="blue" class="w-full">
					<button
						class="w-full"
						:disabled="existingPlan?.id === plansByRam.small.id"
						@click="selectPlan(plansByRam.small!)"
					>
						{{
							existingPlan?.id === plansByRam.small.id
								? formatMessage(messages.yourCurrentPlan)
								: formatMessage(messages.selectPlan)
						}}
					</button>
				</ButtonStyled>
			</div>
			<ServersSpecs
				v-if="smallSpecs"
				:ram="smallSpecs.ram"
				:storage="smallSpecs.storage"
				:cpus="smallSpecs.cpus"
			/>
		</div>

		<!-- Medium (Most Popular) -->
		<div v-if="plansByRam.medium && mediumPrice" class="flex flex-col items-center relative h-full">
			<div
				class="z-10 -mb-3.5 rounded-full text-sm font-medium text-brand whitespace-nowrap absolute -top-3 right-4 bg-surface-3"
			>
				<div
					class="bg-brand-highlight border border-solid border-highlight-green px-2.5 py-0.5 rounded-full"
				>
					{{ formatMessage(messages.mostPopular) }}
				</div>
			</div>

			<div
				class="w-full flex flex-col gap-4 rounded-2xl bg-brand-inverted border-brand-highlight border border-solid p-5 h-full"
				:style="{
					backgroundImage:
						'radial-gradient(86.12% 101.64% at 95.97% 94.07%, rgba(27, 217, 106, 0.23) 0%, rgba(14, 115, 56, 0.2) 100%)',
				}"
			>
				<div>
					<div class="text-3xl font-semibold text-contrast leading-none">
						{{ formatMessage(messages.mediumTitle) }}
					</div>
					<div class="mt-1">
						<span class="text-2xl font-bold text-contrast">
							{{ formatPrice(mediumPrice, currency, true) }}
						</span>
						<span class="text-sm">
							/ month<template v-if="selectedInterval !== 'monthly'"
								>, billed {{ selectedInterval }}</template
							>
						</span>
					</div>
					<div class="mt-2 text-sm text-primary">
						{{ formatMessage(messages.mediumDesc) }}
					</div>
				</div>
				<div class="w-full">
					<ButtonStyled color="brand" class="w-full">
						<button
							class="w-full"
							:disabled="existingPlan?.id === plansByRam.medium.id"
							@click="selectPlan(plansByRam.medium!)"
						>
							{{
								existingPlan?.id === plansByRam.medium.id
									? formatMessage(messages.yourCurrentPlan)
									: formatMessage(messages.selectPlan)
							}}
						</button>
					</ButtonStyled>
				</div>
				<ServersSpecs
					v-if="mediumSpecs"
					:ram="mediumSpecs.ram"
					:storage="mediumSpecs.storage"
					:cpus="mediumSpecs.cpus"
				/>
			</div>
		</div>

		<!-- Large -->
		<div
			v-if="plansByRam.large && largePrice"
			class="flex flex-col gap-4 rounded-2xl bg-surface-2 border-2 border-solid border-transparent p-5 h-full"
		>
			<div>
				<div class="text-3xl font-semibold text-contrast leading-none">
					{{ formatMessage(messages.largeTitle) }}
				</div>
				<div class="mt-1">
					<span class="text-2xl font-bold text-contrast">
						{{ formatPrice(largePrice, currency, true) }}
					</span>
					<span class="text-sm">
						/ month<template v-if="selectedInterval !== 'monthly'"
							>, billed {{ selectedInterval }}</template
						>
					</span>
				</div>
				<div class="mt-2 text-sm text-primary">
					{{ formatMessage(messages.largeDesc) }}
				</div>
			</div>
			<div class="w-full">
				<ButtonStyled color="purple" class="w-full">
					<button
						class="w-full"
						:disabled="existingPlan?.id === plansByRam.large.id"
						@click="selectPlan(plansByRam.large!)"
					>
						{{
							existingPlan?.id === plansByRam.large.id
								? formatMessage(messages.yourCurrentPlan)
								: formatMessage(messages.selectPlan)
						}}
					</button>
				</ButtonStyled>
			</div>
			<ServersSpecs
				v-if="largeSpecs"
				:ram="largeSpecs.ram"
				:storage="largeSpecs.storage"
				:cpus="largeSpecs.cpus"
			/>
		</div>
	</div>

	<!-- Custom plan banner -->
	<div
		v-if="customStartingPrice"
		class="mt-4 flex items-center justify-between gap-4 rounded-2xl bg-surface-2 border-2 border-solid border-transparent p-5"
	>
		<div class="flex flex-col gap-1">
			<div class="text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.customHeading) }}
			</div>
			<div class="text-sm text-secondary">
				{{ formatMessage(messages.customDesc) }}
			</div>
		</div>
		<div class="flex flex-col items-end gap-2 shrink-0">
			<ButtonStyled>
				<button class="flex items-center gap-2" @click="selectCustom">
					{{ formatMessage(messages.getStarted) }} <RightArrowIcon class="h-4 w-4" />
				</button>
			</ButtonStyled>
			<div class="text-sm text-secondary whitespace-nowrap">
				Starting at {{ formatPrice(customStartingPrice, currency, true) }}/mo
			</div>
		</div>
	</div>
</template>
