<script setup lang="ts">
import { formatPrice } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, provide } from 'vue'

import { monthsInInterval, type ServerBillingInterval, type ServerPlan } from '../../utils/billing'
import OptionGroup from '../base/OptionGroup.vue'
import ModalBasedServerPlan from './ModalBasedServerPlan.vue'

const { formatMessage, locale } = useVIntl()

const props = defineProps<{
	availableProducts: ServerPlan[]
	currency: string
	existingPlan?: ServerPlan
}>()

const availableBillingIntervals = ['monthly', 'quarterly']

const selectedPlan = defineModel<ServerPlan>('plan')
const selectedInterval = defineModel<ServerBillingInterval>('interval')
const emit = defineEmits<{
	(e: 'choose-custom'): void
}>()

const messages = defineMessages({
	title: {
		id: 'servers.purchase.step.plan.prompt',
		defaultMessage: 'Choose a plan',
	},
	subtitle: {
		id: 'servers.purchase.step.plan.subtitle',
		defaultMessage: 'Pick the amount of RAM and specs that fit your needs.',
	},
	selectPlan: {
		id: 'servers.purchase.step.plan.select',
		defaultMessage: 'Select Plan',
	},
	getStarted: {
		id: 'servers.purchase.step.plan.get-started',
		defaultMessage: 'Get started',
	},
	billed: {
		id: 'servers.purchase.step.plan.billed',
		defaultMessage: 'billed {interval}',
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
})

const isSameAsExistingPlan = computed(() => {
	return !!(
		props.existingPlan &&
		selectedPlan.value &&
		props.existingPlan.id === selectedPlan.value.id
	)
})

const plansByRam = computed(() => {
	const byName: Record<'small' | 'medium' | 'large', ServerPlan | undefined> = {
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

function handleCustomPlan() {
	emit('choose-custom')
}

function pricePerMonth(plan?: ServerPlan) {
	if (!plan) return undefined
	const total = plan.prices?.find((x) => x.currency_code === props.currency)?.prices?.intervals?.[
		selectedInterval.value!
	]
	if (!total) return undefined
	return total / monthsInInterval[selectedInterval.value!]
}

const customPricePerGb = computed(() => {
	// Calculate lowest price per GB among products for current interval
	let min: number | undefined
	for (const p of props.availableProducts) {
		const perMonth = pricePerMonth(p)
		const ramGb = (p?.metadata?.ram ?? 0) / 1024
		if (perMonth && ramGb > 0) {
			const perGb = perMonth / ramGb
			if (min === undefined || perGb < min) min = perGb
		}
	}
	return min
})

const customStartingPrice = computed(() => {
	let min: number | undefined
	for (const p of props.availableProducts) {
		const perMonth = pricePerMonth(p)
		if (perMonth && (min === undefined || perMonth < min)) min = perMonth
	}
	return min
})

provide('currency', props.currency)
provide('selectedInterval', selectedInterval)
</script>

<template>
	<div class="grid grid-cols-[1fr_auto_1fr] items-center gap-3 mb-5 !mt-0">
		<span></span>
		<OptionGroup
			v-slot="{ option }"
			v-model="selectedInterval"
			class="!bg-button-bg !shadow-none"
			:options="availableBillingIntervals"
		>
			<template v-if="option === 'monthly'"> Pay monthly </template>
			<span v-else-if="option === 'quarterly'"> Pay quarterly </span>
			<span v-else-if="option === 'yearly'"> Pay yearly </span>
		</OptionGroup>
		<span class="bg-transparent p-0 text-sm text-xs font-bold text-brand">
			{{ selectedInterval !== 'quarterly' ? 'Save' : 'Saving' }} 16% with quarterly billing!
		</span>
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
	<div class="grid grid-cols-1 sm:grid-cols-2 !gap-4">
		<ModalBasedServerPlan
			v-if="plansByRam.small"
			:plan="plansByRam.small"
			:title="{ id: 'servers.purchase.step.plan.small', defaultMessage: 'Small' }"
			:description="messages.smallDesc"
			:button-color="'blue'"
			:selected="selectedPlan?.id === plansByRam.small.id"
			@select="selectedPlan = $event"
		/>
		<ModalBasedServerPlan
			v-if="plansByRam.medium"
			:plan="plansByRam.medium"
			:title="{ id: 'servers.purchase.step.plan.medium', defaultMessage: 'Medium' }"
			:description="messages.mediumDesc"
			most-popular
			:button-color="'brand'"
			:selected="selectedPlan?.id === plansByRam.medium.id"
			@select="selectedPlan = $event"
		/>
		<ModalBasedServerPlan
			v-if="plansByRam.large"
			:plan="plansByRam.large"
			:title="{ id: 'servers.purchase.step.plan.large', defaultMessage: 'Large' }"
			:description="messages.largeDesc"
			:button-color="'purple'"
			:selected="selectedPlan?.id === plansByRam.large.id"
			@select="selectedPlan = $event"
		/>
		<div
			v-if="customStartingPrice"
			class="rounded-2xl p-4 font-semibold transition-all duration-300 experimental-styles-within h-full border-2 border-solid cursor-pointer select-none"
			:class="!selectedPlan ? 'bg-brand-highlight border-brand' : 'bg-button-bg border-transparent'"
			role="button"
			tabindex="0"
			:aria-pressed="!selectedPlan"
			@click="handleCustomPlan"
			@keydown.enter.prevent="handleCustomPlan"
			@keydown.space.prevent="handleCustomPlan"
		>
			<div class="flex h-full flex-col justify-between">
				<div class="flex flex-col gap-2">
					<div class="flex items-center justify-between">
						<span class="text-2xl font-semibold text-contrast">Custom</span>
					</div>
					<span class="m-0 text-lg font-bold text-contrast">
						{{ formatPrice(locale, customStartingPrice, currency, true) }}
						<span class="text-sm font-semibold text-secondary">
							/ month<template v-if="selectedInterval !== 'monthly'"
								>, billed {{ selectedInterval }}</template
							>
						</span>
					</span>
					<span class="text-sm">{{ formatMessage(messages.customDesc) }}</span>
				</div>
				<div class="flex flex-col gap-2">
					<div class="flex items-center gap-3">
						<span v-if="customPricePerGb" class="text-sm text-secondary">
							From {{ formatPrice(locale, customPricePerGb, currency, true) }} / GB
						</span>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>
