<script setup lang="ts">
import { ButtonStyled, ServersSpecs } from '@modrinth/ui'
import { formatPrice } from '@modrinth/utils'
import type { MessageDescriptor } from '@vintl/vintl'

const { formatMessage, locale } = useVIntl()

const emit = defineEmits<{
	(e: 'select' | 'scroll-to-faq'): void
}>()

type Plan = 'small' | 'medium' | 'large'

const plans: Record<
	Plan,
	{
		buttonColor: 'blue' | 'green' | 'purple'
		accentText: string
		accentBg: string
		name: MessageDescriptor
		description: MessageDescriptor
		mostPopular: boolean
	}
> = {
	small: {
		buttonColor: 'blue',
		accentText: 'text-blue',
		accentBg: 'bg-bg-blue',
		name: defineMessage({
			id: 'servers.plan.small.name',
			defaultMessage: 'Small',
		}),
		description: defineMessage({
			id: 'servers.plan.small.description',
			defaultMessage: 'Perfect for 1–5 friends with a few light mods.',
		}),
		mostPopular: false,
	},
	medium: {
		buttonColor: 'green',
		accentText: 'text-green',
		accentBg: 'bg-bg-green',
		name: defineMessage({
			id: 'servers.plan.medium.name',
			defaultMessage: 'Medium',
		}),
		description: defineMessage({
			id: 'servers.plan.medium.description',
			defaultMessage: 'Great for 6–15 players and multiple mods.',
		}),
		mostPopular: true,
	},
	large: {
		buttonColor: 'purple',
		accentText: 'text-purple',
		accentBg: 'bg-bg-purple',
		name: defineMessage({
			id: 'servers.plan.large.name',
			defaultMessage: 'Large',
		}),
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
						Most popular
					</div>
				</div>
				<span class="m-0 text-2xl font-bold text-contrast">
					{{ formatPrice(locale, price / billingMonths, currency, true) }}
					{{ isUsa ? '' : currency }}
					<span class="text-lg font-semibold text-secondary">
						/ month<template v-if="interval !== 'monthly'">, billed {{ interval }}</template>
					</span>
				</span>
				<p class="m-0 max-w-[18rem]">{{ formatMessage(plans[plan].description) }}</p>
			</div>
			<ButtonStyled
				:color="plans[plan].buttonColor"
				:type="plans[plan].mostPopular ? 'standard' : 'highlight-colored-text'"
				size="large"
			>
				<span v-if="outOfStock" class="button-like disabled"> Out of Stock </span>
				<button v-else @click="() => emit('select')">Select plan</button>
			</ButtonStyled>
			<ServersSpecs
				:ram="ram"
				:storage="storage"
				:cpus="cpus"
				:bursting-link="'/servers#cpu-burst'"
				@click-bursting-link="() => emit('scroll-to-faq')"
			/>
		</div>
	</li>
</template>

<style scoped lang="scss"></style>
