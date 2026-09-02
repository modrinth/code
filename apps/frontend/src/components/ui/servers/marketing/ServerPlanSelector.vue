<script setup lang="ts">
import { type MessageDescriptor, useFormatPrice } from '@modrinth/ui'
import {
	Button,
	commonMessages,
	defineMessage,
	defineMessages,
	ServersSpecs,
	useVIntl,
} from '@modrinth/ui'

const { formatMessage } = useVIntl()
const formatPrice = useFormatPrice()

const emit = defineEmits<{
	(e: 'select' | 'scroll-to-faq'): void
}>()

type Plan = 'small' | 'medium' | 'large'

const messages = defineMessages({
	outOfStock: {
		id: 'hosting.plan.out-of-stock',
		defaultMessage: 'Out of stock',
	},
	selectPlanButton: {
		id: 'hosting.plan.select-plan',
		defaultMessage: 'Select plan',
	},
	perMonthBilling: {
		id: 'servers.purchase.step.plan.per-month-billing',
		defaultMessage:
			'/ month{interval, select, monthly {} other {, billed {interval, select, quarterly {quarterly} yearly {yearly} other {{interval}}}}}',
	},
	mostPopularBadge: {
		id: 'servers.purchase.step.plan.most-popular',
		defaultMessage: 'Most popular',
	},
})

const plans: Record<
	Plan,
	{
		buttonColor: 'blue' | 'green' | 'purple'
		buttonClasses: string
		name: MessageDescriptor
		description: MessageDescriptor
		mostPopular: boolean
	}
> = {
	small: {
		buttonColor: 'blue',
		buttonClasses: '!bg-highlight-blue !text-blue',
		name: commonMessages.planSmallLabel,
		description: defineMessage({
			id: 'servers.plan.small.description',
			defaultMessage: 'Perfect for 1–5 friends with a few light mods.',
		}),
		mostPopular: false,
	},
	medium: {
		buttonColor: 'green',
		buttonClasses: '',
		name: commonMessages.planMediumLabel,
		description: defineMessage({
			id: 'servers.plan.medium.description',
			defaultMessage: 'Great for 6–15 players and multiple mods.',
		}),
		mostPopular: true,
	},
	large: {
		buttonColor: 'purple',
		buttonClasses: '!bg-highlight-purple !text-purple',
		name: commonMessages.planLargeLabel,
		description: defineMessage({
			id: 'servers.plan.large.description',
			defaultMessage: 'Ideal for 15–25 players, modpacks, or heavy modding.',
		}),
		mostPopular: false,
	},
}

const props = defineProps<{
	capacity?: number
	plan: Plan
	ram: number
	storage: number
	cpus: number
	price: number
	interval: 'monthly' | 'quarterly' | 'yearly'
	currency: string
	isUsa: boolean
}>()

const outOfStock = computed(() => {
	return !props.capacity || props.capacity === 0
})

const billingMonths = computed(() => {
	if (props.interval === 'yearly') {
		return 12
	} else if (props.interval === 'quarterly') {
		return 3
	}
	return 1
})
</script>

<template>
	<li class="relative flex w-full flex-col justify-between">
		<div
			:style="
				plans[plan].mostPopular
					? {
							background: `radial-gradient(
                  86.12% 101.64% at 95.97% 94.07%,
                  rgba(27, 217, 106, 0.23) 0%,
                  rgba(14, 115, 56, 0.2) 100%
                )`,
							border: `1px solid rgba(12, 107, 52, 0.55)`,
							'box-shadow': `0px 12px 38.1px rgba(27, 217, 106, 0.13)`,
						}
					: undefined
			"
			class="flex w-full flex-col justify-between gap-4 rounded-2xl bg-bg p-8 text-left"
		>
			<div class="flex flex-col gap-2">
				<div class="flex flex-row flex-wrap items-center gap-3">
					<h1 class="m-0">{{ formatMessage(plans[plan].name) }}</h1>
					<div
						v-if="plans[plan].mostPopular"
						class="rounded-full bg-brand-highlight px-2 py-1 text-xs font-bold text-brand"
					>
						{{ formatMessage(messages.mostPopularBadge) }}
					</div>
				</div>
				<span class="m-0 text-2xl font-bold text-contrast">
					{{ formatPrice(price / billingMonths, currency, true) }}
					<span class="text-lg font-semibold text-secondary">
						{{ formatMessage(messages.perMonthBilling, { interval }) }}
					</span>
				</span>
				<p class="m-0 max-w-[18rem]">{{ formatMessage(plans[plan].description) }}</p>
			</div>
			<Button
				v-if="outOfStock"
				disabled
				size="xl"
				:type="plans[plan].mostPopular ? 'colored' : 'quiet'"
				:color="plans[plan].buttonColor"
				:interaction="plans[plan].mostPopular ? undefined : 'none'"
				:class="plans[plan].buttonClasses"
			>
				{{ formatMessage(messages.outOfStock) }}
			</Button>
			<Button
				v-else
				size="xl"
				:type="plans[plan].mostPopular ? 'colored' : 'quiet'"
				:color="plans[plan].buttonColor"
				:interaction="plans[plan].mostPopular ? undefined : 'none'"
				:class="plans[plan].buttonClasses"
				@click="() => emit('select')"
			>
				{{ formatMessage(messages.selectPlanButton) }}
			</Button>
			<ServersSpecs
				:ram="ram"
				:storage="storage"
				:cpus="cpus"
				:bursting-link="'/hosting#cpu-burst'"
				@click-bursting-link="() => emit('scroll-to-faq')"
			/>
		</div>
	</li>
</template>

<style scoped lang="scss"></style>
